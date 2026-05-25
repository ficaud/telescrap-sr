use crate::core::scan::ScanConfig;
use crate::core::app_state::AppState;
use crate::controller::notify::Notify;
use crate::app::scan_task::ScanTask;
use tokio::sync::watch;

pub struct ScannerHandle {
    abort_handle: tokio::task::AbortHandle,
}

impl ScannerHandle {
    pub fn configure() -> ScanConfig {
        ScanConfig::default()
    }
    
    pub fn start(config_rx: watch::Receiver<ScanConfig>, notifier: impl Notify, state_rx: watch::Receiver<AppState>) -> Self {
        let task = ScanTask::new(config_rx, notifier, state_rx);
        let handle = tokio::spawn(async move { task.run().await });
        Self { abort_handle: handle.abort_handle() }
    }

    pub fn stop(&self) {
        self.abort_handle.abort();
    }
}