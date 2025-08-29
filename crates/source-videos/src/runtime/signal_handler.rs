use crate::error::{Result, SourceVideoError};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum SignalEvent {
    Reload,
    Shutdown,
}

pub struct SignalHandler {
    tx: mpsc::Sender<SignalEvent>,
    rx: Option<mpsc::Receiver<SignalEvent>>,
}

impl SignalHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(10);
        Self { tx, rx: Some(rx) }
    }

    pub async fn start(mut self) -> Result<mpsc::Receiver<SignalEvent>> {
        let tx = self.tx.clone();

        // Spawn signal handler for Unix systems
        #[cfg(unix)]
        {
            tokio::spawn(async move {
                use tokio::signal::unix::{SignalKind, signal};

                let mut sighup =
                    signal(SignalKind::hangup()).expect("Failed to create SIGHUP handler");
                let mut sigterm =
                    signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
                let mut sigint =
                    signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");

                loop {
                    tokio::select! {
                        _ = sighup.recv() => {
                            log::info!("Received SIGHUP - triggering configuration reload");
                            let _ = tx.send(SignalEvent::Reload).await;
                        }
                        _ = sigterm.recv() => {
                            log::info!("Received SIGTERM - initiating shutdown");
                            let _ = tx.send(SignalEvent::Shutdown).await;
                            break;
                        }
                        _ = sigint.recv() => {
                            log::info!("Received SIGINT - initiating shutdown");
                            let _ = tx.send(SignalEvent::Shutdown).await;
                            break;
                        }
                    }
                }
            });
        }

        // Spawn signal handler for Windows systems
        #[cfg(windows)]
        {
            tokio::spawn(async move {
                use tokio::signal::windows;

                let mut ctrl_c = windows::ctrl_c().expect("Failed to create Ctrl+C handler");
                let mut ctrl_break =
                    windows::ctrl_break().expect("Failed to create Ctrl+Break handler");

                loop {
                    tokio::select! {
                        _ = ctrl_c.recv() => {
                            log::info!("Received Ctrl+C - initiating shutdown");
                            let _ = tx.send(SignalEvent::Shutdown).await;
                            break;
                        }
                        _ = ctrl_break.recv() => {
                            log::info!("Received Ctrl+Break - triggering configuration reload");
                            let _ = tx.send(SignalEvent::Reload).await;
                        }
                    }
                }
            });
        }

        self.rx
            .take()
            .ok_or_else(|| SourceVideoError::resource("Signal receiver already taken"))
    }

    pub fn trigger_reload(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(SignalEvent::Reload).await;
        });
    }

    pub fn trigger_shutdown(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(SignalEvent::Shutdown).await;
        });
    }
}

pub async fn setup_signal_handlers() -> Result<mpsc::Receiver<SignalEvent>> {
    let handler = SignalHandler::new();
    handler.start().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        let mut rx = handler.start().await.unwrap();

        // Should not receive any signal immediately
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_err()); // Timeout expected
    }

    #[tokio::test]
    async fn test_manual_trigger() {
        let handler = SignalHandler::new();
        let tx_clone = handler.tx.clone();
        let mut rx = handler.start().await.unwrap();

        // Use the cloned tx to trigger reload
        tokio::spawn(async move {
            let _ = tx_clone.send(SignalEvent::Reload).await;
        });

        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("Timeout")
            .expect("No event received");

        assert!(matches!(event, SignalEvent::Reload));
    }
}
