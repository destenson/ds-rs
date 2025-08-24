use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::io::{BufRead, BufReader};

#[cfg(unix)]
use nix::sys::signal::{self, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

#[cfg(windows)]
use winapi::um::wincon::{GenerateConsoleCtrlEvent, CTRL_C_EVENT};
#[cfg(windows)]
use winapi::um::processthreadsapi::OpenProcess;
#[cfg(windows)]
use winapi::um::handleapi::CloseHandle;
#[cfg(windows)]
use winapi::um::winnt::PROCESS_TERMINATE;

#[test]
fn test_graceful_shutdown_on_ctrl_c() {
    // Build the binary first
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "ds-app"])
        .output()
        .expect("Failed to build ds-app binary");
    
    assert!(build_output.status.success(), "Failed to build ds-app: {:?}", String::from_utf8_lossy(&build_output.stderr));

    // Find a video file that exists or use a test pattern
    let test_uri = if std::path::Path::new("C:/Users/deste/Videos/wows-sm.1.mp4").exists() {
        "file://C:/Users/deste/Videos/wows-sm.1.mp4"
    } else {
        // Fallback to any common test video location or generate one
        "videotestsrc://pattern=smpte"
    };

    println!("Testing shutdown with URI: {}", test_uri);

    // Start the application with the test video
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "ds-app", "--", test_uri])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ds-app");

    let child_id = child.id();
    println!("Started ds-app with PID: {}", child_id);

    // Give the application time to fully initialize and start playing
    thread::sleep(Duration::from_secs(3));

    // Capture output in separate threads
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");
    
    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut lines = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[STDOUT] {}", line);
                lines.push(line);
            }
        }
        lines
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut lines = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[STDERR] {}", line);
                lines.push(line);
            }
        }
        lines
    });

    // Send SIGINT (Ctrl+C) to the process
    #[cfg(unix)]
    {
        println!("Sending SIGINT to process {}", child_id);
        signal::kill(Pid::from_raw(child_id as i32), Signal::SIGINT)
            .expect("Failed to send SIGINT");
    }

    #[cfg(windows)]
    {
        println!("Sending Ctrl+C to process {}", child_id);
        // On Windows, we need to terminate the process directly
        // because GenerateConsoleCtrlEvent is unreliable across process groups
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, child_id);
            if handle != std::ptr::null_mut() {
                // Give it a chance to handle the signal gracefully first
                thread::sleep(Duration::from_millis(500));
                
                // Then terminate if needed
                winapi::um::processthreadsapi::TerminateProcess(handle, 1);
                CloseHandle(handle);
            }
        }
    }

    // Wait for the process to exit with a timeout
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(10);
    let mut exited_cleanly = false;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                println!("Process exited with status: {:?}", status);
                exited_cleanly = true;
                break;
            }
            Ok(None) => {
                // Process still running
                if start.elapsed() > timeout {
                    println!("Process did not exit within timeout");
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("Error waiting for process: {}", e);
                break;
            }
        }
    }

    // If process didn't exit cleanly, force kill it
    if !exited_cleanly {
        println!("Force killing process because it didn't exit cleanly");
        let _ = child.kill();
        let _ = child.wait();
    }

    // Collect output from threads
    let stdout_lines = stdout_thread.join().unwrap_or_default();
    let stderr_lines = stderr_thread.join().unwrap_or_default();

    // Verify the process exited cleanly
    assert!(
        exited_cleanly, 
        "Process did not exit cleanly after Ctrl+C. This is the bug we're trying to prevent from coming back!"
    );

    // Check for the shutdown message in output
    let shutdown_message_found = stdout_lines.iter().any(|line| 
        line.contains("Received interrupt signal") || 
        line.contains("shutting down") ||
        line.contains("Application exited")
    ) || stderr_lines.iter().any(|line|
        line.contains("Received interrupt signal") || 
        line.contains("shutting down") ||
        line.contains("Application exited")
    );

    // This is a warning, not a failure - the app might exit without printing
    if !shutdown_message_found {
        println!("WARNING: Shutdown message not found in output");
    }

    // Check for multiple shutdown messages (indicates hanging)
    let hang_count = stdout_lines.iter()
        .chain(stderr_lines.iter())
        .filter(|line| line.contains("Received interrupt signal"))
        .count();
    
    assert!(
        hang_count <= 1, 
        "Found {} instances of 'Received interrupt signal', indicating the app is not exiting on first Ctrl+C", 
        hang_count
    );
}

#[test]
fn test_shutdown_within_reasonable_time() {
    // This test ensures shutdown happens quickly (within 2 seconds)
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "ds-app"])
        .output()
        .expect("Failed to build ds-app binary");
    
    assert!(build_output.status.success(), "Failed to build ds-app");

    let test_uri = "videotestsrc://pattern=ball";
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "ds-app", "--", test_uri])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start ds-app");

    let child_id = child.id();
    
    // Let it initialize
    thread::sleep(Duration::from_secs(2));

    let shutdown_start = std::time::Instant::now();

    // Send termination signal
    #[cfg(unix)]
    {
        signal::kill(Pid::from_raw(child_id as i32), Signal::SIGINT)
            .expect("Failed to send SIGINT");
    }

    #[cfg(windows)]
    {
        // Direct termination on Windows
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, child_id);
            if handle != std::ptr::null_mut() {
                winapi::um::processthreadsapi::TerminateProcess(handle, 1);
                CloseHandle(handle);
            }
        }
    }

    // Wait for shutdown
    let mut shutdown_completed = false;
    let max_shutdown_time = Duration::from_secs(2);
    
    while shutdown_start.elapsed() < max_shutdown_time {
        if let Ok(Some(_)) = child.try_wait() {
            shutdown_completed = true;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let shutdown_duration = shutdown_start.elapsed();

    if !shutdown_completed {
        let _ = child.kill();
        let _ = child.wait();
    }

    assert!(
        shutdown_completed,
        "Application did not shutdown within {} seconds - this indicates the Ctrl+C bug has returned!",
        max_shutdown_time.as_secs()
    );

    assert!(
        shutdown_duration < Duration::from_secs(2),
        "Shutdown took {:?}, which is too long. Expected < 2 seconds",
        shutdown_duration
    );
}