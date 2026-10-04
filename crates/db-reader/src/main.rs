use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

const ENCOUNTERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("encounters");

#[derive(Debug, Serialize, Deserialize)]
struct EncounterRecord {
    stable_id: String,
    title: String,
    date: String,
    club_type: String,
    resale_link: String,
    resale_active: bool,
}

fn init_logger() {
    use std::io::Write;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let now = chrono::Utc::now().with_timezone(&chrono_tz::Europe::Paris);
            writeln!(
                buf,
                "[{} {}] {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}

fn main() {
    init_logger();

    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .cloned()
        .or_else(|| std::env::var("MATCHS_DB_PATH").ok())
        .unwrap_or_else(|| "matchs.db".to_string());

    let db = match Database::open(&path) {
        Ok(db) => db,
        Err(e) => {
            log::error!("Failed to open database '{}': {}", path, e);
            log::error!("");
            log::error!("Usage: db-reader [path]  (defaults to MATCHS_DB_PATH or 'matchs.db')");
            std::process::exit(1);
        }
    };

    let txn = db.begin_read().expect("Failed to begin read transaction");
    let table = txn
        .open_table(ENCOUNTERS_TABLE)
        .expect("Failed to open encounters table");

    let mut records: Vec<EncounterRecord> = Vec::new();
    for entry in table.iter().expect("Failed to iterate table") {
        let item = entry.expect("Failed to read entry");
        match serde_json::from_str::<EncounterRecord>(item.1.value()) {
            Ok(record) => records.push(record),
            Err(e) => log::warn!("Warning: failed to deserialize record: {}", e),
        }
    }

    if records.is_empty() {
        println!("No records found in '{}'.", path);
        return;
    }

    println!(
        "{:<45} {:<12} {:<10} {:<12} {}",
        "Title", "Date", "Active", "Club type", "Resale link"
    );
    println!("{}", "-".repeat(120));
    for r in &records {
        println!(
            "{:<45} {:<12} {:<10} {:<12} {}",
            truncate(&r.title, 44),
            r.date,
            if r.resale_active { "yes" } else { "no" },
            r.club_type,
            r.resale_link,
        );
    }
    println!("\nTotal: {} record(s)", records.len());
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
