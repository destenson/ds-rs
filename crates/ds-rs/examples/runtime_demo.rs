use ds_rs::{PlatformInfo, init};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init()?;

    println!("DeepStream Rust - Runtime Demo Example");
    println!("======================================\n");

    let platform = PlatformInfo::detect()?;
    println!("Platform: {:?}", platform.platform);
    println!("Has NVIDIA Hardware: {}", platform.has_nvidia_hardware());

    println!("\nTo run the full runtime demo:");
    println!("  cargo run --bin ds-app -- <video_uri>");
    println!("\nExample URIs:");
    println!("  file:///path/to/video.mp4");
    println!("  rtsp://camera.local/stream");
    println!("  videotestsrc pattern=smpte ! video/x-raw,width=640,height=480");

    println!("\nThe demo will:");
    println!("  1. Start with one source");
    println!("  2. Add a new source every 10 seconds");
    println!("  3. After reaching 4 sources, start removing them");
    println!("  4. Continue until all sources are removed");

    Ok(())
}
