
use parser::core::encounter::Encounter;
/// This module defines the `ScanTask` struct and its associated logic for performing periodic scans of encounters,
/// applying filters, and notifying about changes in available seats.
use parser::interface::curl::proxy::set_proxy_enabled;
use parser::interface::match_manager;
use filter::filter::Filter;
use std::panic::{AssertUnwindSafe, catch_unwind};
use tokio::sync::watch;
use tokio::time::{interval, Duration};
use crate::{
    app::diff::{diff, DiffType},
    controller::notify::Notify,
    core::app_state::AppState,
    core::scan::{ScanConfig, ScanMode, ScanResult},
};


/// Represents a scanning task that periodically checks for changes in encounters based on a specified configuration,
/// applies filters to the results, and sends notifications about any detected changes.
pub struct ScanTask<N: Notify> {
    config: ScanConfig,
    config_rx: watch::Receiver<ScanConfig>,
    notifier: N,
    previous: Option<ScanResult>,
    state_rx: watch::Receiver<AppState>,
}

impl<N: Notify> ScanTask<N> {
    /// Creates a new `ScanTask` with the specified configuration and notifier.
    ///
    /// # Arguments
    /// * `config` - The configuration for the scan task.
    /// * `notifier` - The notifier to use for sending notifications about changes.
    ///
    /// # Returns
    /// A new instance of `ScanTask` initialized with the provided configuration and notifier.
    pub fn new(mut config_rx: watch::Receiver<ScanConfig>, notifier: N, state_rx: watch::Receiver<AppState>) -> Self {
        let config = config_rx.borrow_and_update().clone();
        set_proxy_enabled(config.proxy_enabled);
        Self { config, config_rx, notifier, previous: None, state_rx }
    }

    /// Runs the scan task, periodically checking for changes in encounters, applying filters,
    /// and sending notifications about any detected changes.
    pub async fn run(mut self) {
        let mut ticker = interval(Duration::from_secs(self.config.interval));
        loop {
            tokio::select! {
                _ = ticker.tick() => {}
                result = self.config_rx.changed() => {
                    if result.is_err() {
                        break; // sender dropped (shutdown), exit cleanly
                    }
                    self.config = self.config_rx.borrow_and_update().clone();
                    set_proxy_enabled(self.config.proxy_enabled);
                    self.previous = None;
                    ticker = interval(Duration::from_secs(self.config.interval));
                    println!("⚙️  Configuration mise à jour, redémarrage du cycle");
                    continue;
                }

                result = self.state_rx.changed() => {
                    if result.is_err() {
                        break; // sender dropped (shutdown), exit cleanly
                    }
                    println!("🔄 État de l'application changé : {:?}", *self.state_rx.borrow());
                    continue;
                }
            }

            if *self.state_rx.borrow() == AppState::Stopped {
                continue;
            }

            // variable that will be used to measure the duration of the scan process
            let scan_start = std::time::Instant::now();
            // Fetch current encounters from the match manager in a blocking task to avoid blocking the async runtime
            let club = self.config.club.clone();
            // Fetch match nature from config
            let nature = self.config.nature;
            // Check if a specific match title filter is set
            let filter_title = self.config.filter_chain
                .as_ref()
                .and_then(|c| c.encounter_title())
                .map(|s| s.to_string());

            // Fetch encounters from the match manager in a blocking task to avoid blocking the async runtime
            let scan_result = tokio::task::spawn_blocking(move || {
                let encounters = if let Some(ref title) = filter_title {
                    match_manager::get_seats_from_match_title(title.clone(), club, nature)
                } else {
                    match_manager::get_seats_from_matches(club, nature)
                };
                println!("[SCAN_TASK] {} encounter(s) retrieved", encounters.len());
                ScanResult::new(encounters)
            })
            .await
            .unwrap();

            // Compare with previous results to detect changes (new seats)
            let changed: Vec<Encounter> = if let Some(prev) = &self.previous {
                diff(&prev.encounters, &scan_result.encounters)
                    .into_iter()
                    .filter(|r| r.diff_type == DiffType::NewSeats)
                    .map(|r| r.encounter_diff_only)
                    .collect()
            } else {
                // First iteration: treat available seats as new
                scan_result.encounters.iter()
                    .filter(|e| e.seats.as_ref().map_or(false, |s| !s.is_empty()))
                    .cloned()
                    .collect()
            };

            // Apply the filter chain from config (built by admin panel)
            let result = if let Some(chain) = &self.config.filter_chain {
                chain.apply(&changed)
            } else {
                changed
            };

            // Load seat preview images if enabled
            let mut result = result;
            if self.config.is_preview {
                for encounter in &mut result {
                    if let Some(seats) = &mut encounter.seats {
                        for seat in seats.iter_mut() {
                            seat.seat_info.preview_url = match_manager::get_seat_preview(
                                &self.config.club,
                                &seat.seat_info.composition,
                            );
                        }
                    }
                }
            }

            // In aggressive mode, attempt to add detected seats to the basket on the ticketing website.
            let basket_successes = if self.config.mode == ScanMode::AggressiveScan {
                self.try_aggressive_add_to_basket(&result)
            } else {
                0
            };

            // Calculate elapsed time and send notifications if there are changes, otherwise log that no change was detected
            if !result.is_empty() {
                let elapsed = scan_start.elapsed();
                println!("⚠️  {} changement(s) détecté(s) ({:.2?})", result.len(), elapsed);
                dbg!(&result);
                self.notify_parsed_info(&result, basket_successes);
            } else {
                #[allow(unused_variables)]
                let elapsed = scan_start.elapsed();
                // println!("✅ Aucun changement détecté ({:.2?})", elapsed);
            }

            self.previous = Some(scan_result);
        }
    }

    /// In aggressive mode, attempts to automatically add detected seats to the basket on the ticketing website.
    /// This method uses the `match_manager` to connect to the ticketing website and add seats to the cart based on the provided encounters.
    ///
    /// # Arguments
    /// * `encounters` - A slice of `Encounter` instances representing the detected encounters with available seats that should be added to the basket.
    /// # Returns
    /// The number of seats that were successfully added to the basket.
    ///
    /// Note: This method requires the `SHOP_EMAIL` and `SHOP_PASSWORD` environment variables to be set with valid credentials for the ticketing website.
    fn try_aggressive_add_to_basket(&self, encounters: &[Encounter]) -> usize {
        if encounters.is_empty() {
            return 0;
        }

        let email = match std::env::var("SHOP_EMAIL") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                eprintln!("[AGGRESSIVE] SHOP_EMAIL missing, auto add-to-basket skipped");
                return 0;
            }
        };
        let password = match std::env::var("SHOP_PASSWORD") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                eprintln!("[AGGRESSIVE] SHOP_PASSWORD missing, auto add-to-basket skipped");
                return 0;
            }
        };

        let mut attempts = 0usize;
        let mut successes = 0usize;

        for encounter in encounters {
            if let Some(seats) = &encounter.seats {
                for seat in seats {
                    attempts += 1;
                    let add_result = catch_unwind(AssertUnwindSafe(|| {
                        match_manager::connect_and_add_seat_to_cart(
                            email.clone(),
                            password.clone(),
                            seat.clone(),
                        )
                    }));

                    match add_result {
                        Ok(Ok(())) => {
                            successes += 1;
                            println!(
                                "[AGGRESSIVE] Seat added to basket: {} | {}",
                                encounter.title,
                                seat.seat_info.full_name
                            );
                        }
                        Ok(Err(err)) => {
                            eprintln!(
                                "[AGGRESSIVE] Add-to-basket failed for '{}' seat '{}': {}",
                                encounter.title,
                                seat.seat_info.full_name,
                                err
                            );
                        }
                        Err(_) => {
                            eprintln!(
                                "[AGGRESSIVE] Add-to-basket panic for '{}' seat '{}' (implementation likely incomplete)",
                                encounter.title,
                                seat.seat_info.full_name,
                            );
                        }
                    }
                }
            }
        }

        println!(
            "[AGGRESSIVE] Basket attempts: {}, successes: {}",
            attempts,
            successes
        );

        successes
    }

    /// Notifies about the parsed information by constructing a message that includes the details of the encounters and the detected changes, and sending it through the notifier.
    ///
    /// # Arguments
    /// * `changed` - A slice of `DiffResult` instances representing the detected changes that should be included in the notification.
    ///
    /// # Returns
    /// This method does not return a value, but it sends a formatted message through the notifier containing the details of the encounters and the detected changes.
    fn notify_parsed_info(&self, changed: &[Encounter], basket_successes: usize) {
        if basket_successes > 0 {
            self.notifier.send(&format!(
                "🧺 <b>{} ticket(s) ajouté(s) au panier.</b>\nConnecte-toi à la billetterie pour finaliser l'achat.",
                basket_successes
            ));
        }

        let header = format!("🏉 <b>{}</b>", self.config.club.name);

        for encounter in changed {
            let resale = match &encounter.resale_link {
                Some(link) => format!("\n🔗 <a href=\"{}\">Accéder à la revente</a>", link),
                None => String::new(),
            };

            let encounter_header = format!(
                "{}\n\n━━━━━━━━━━━━━━━━\n\n🆚 <b>{}</b>\n📅 <i>{}</i>\n\n🟢 <b>Nouvelles places :</b>{}",
                header, encounter.title, encounter.date, resale,
            );

            match &encounter.seats {
                Some(seats) if !seats.is_empty() => {
                    // Seats with a preview: one photo message per seat
                    let (with_preview, without_preview): (Vec<_>, Vec<_>) = seats.iter()
                        .partition(|s| s.seat_info.preview_url.is_some());

                    for seat in &with_preview {
                        let category = seat.seat_info.composition.category.as_str();
                        let full_name = seat.seat_info.full_name.as_str();
                        let price = seat.price.as_str();
                        let seat_line = if category.is_empty() {
                            format!("  • {} — <code>{}€ </code>", full_name, price)
                        } else {
                            format!("  • [{}] {} — <code>{}€ </code>", category, full_name, price)
                        };
                        let caption = format!("{}\n\n{}", encounter_header, seat_line);
                        self.notifier.send_photo(seat.seat_info.preview_url.as_deref().unwrap(), &caption);
                    }

                    // Remaining seats without preview: one grouped text message
                    if !without_preview.is_empty() {
                        let seat_list = without_preview.iter()
                            .map(|s| {
                                let category = s.seat_info.composition.category.as_str();
                                let full_name = s.seat_info.full_name.as_str();
                                let price = s.price.as_str();
                                if category.is_empty() {
                                    format!("  • {} — <code>{}€ </code>", full_name, price)
                                } else {
                                    format!("  • [{}] {} — <code>{}€ </code>", category, full_name, price)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        self.notifier.send(&format!("{}\n\n{}", encounter_header, seat_list));
                    }
                }
                _ => {
                    self.notifier.send(&format!("{}\n\n  <i>Aucun siège disponible</i>", encounter_header));
                }
            }
        }
    }
}
