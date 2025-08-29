use crate::error::Result;
use crate::pipeline::Pipeline;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run_main_loop(
    pipeline: Arc<Pipeline>,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<()> {
    let bus = pipeline.bus().ok_or_else(|| {
        crate::error::DeepStreamError::Pipeline("No bus available on pipeline".to_string())
    })?;

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                println!("Received shutdown signal");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(10)) {
                    match msg.view() {
                        gst::MessageView::Eos(..) => {
                            println!("End of stream");
                            break;
                        }
                        gst::MessageView::Error(err) => {
                            eprintln!(
                                "Error from {:?}: {} ({:?})",
                                err.src().map(|s| s.path_string()),
                                err.error(),
                                err.debug()
                            );
                            break;
                        }
                        gst::MessageView::Warning(warning) => {
                            eprintln!(
                                "Warning from {:?}: {} ({:?})",
                                warning.src().map(|s| s.path_string()),
                                warning.error(),
                                warning.debug()
                            );
                        }
                        gst::MessageView::StreamStatus(status) => {
                            if let Some(obj) = status.stream_status_object() {
                                println!(
                                    "Stream status: {:?} from {:?}",
                                    obj.type_(),
                                    status.src().map(|s| s.path_string())
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
