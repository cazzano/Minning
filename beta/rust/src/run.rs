use std::process::{Command, Stdio, Child};
use std::env;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::fs;

// The original function - kept for backward compatibility
pub fn run_xmr() -> io::Result<()> {
    // Get home directory and create the full path
    let home_dir = env::var("HOME").expect("Failed to get HOME environment variable");
    let xmr_path = format!("{}/xmr/xmr", home_dir);
    
    // First command: chmod +x /home/user/xmr/xmr
    let chmod_status = Command::new("chmod")
        .arg("+x")
        .arg(&xmr_path)
        .status()?;

    if !chmod_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other, 
            "chmod command failed"
        ));
    }

    // Second command: execute /home/user/xmr/xmr
    let run_status = Command::new(&xmr_path)
        .status()?;

    if !run_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other, 
            "xmr execution failed"
        ));
    }

    Ok(())
}

// Function to capture and handle CTRL+C
fn setup_ctrlc_handler() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("Received CTRL+C, preparing for graceful shutdown...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    running
}

// Function to create a watchdog that restarts the process if it's killed
fn create_watchdog(xmr_path: String, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut current_process: Option<Child> = None;
        
        while running.load(Ordering::SeqCst) {
            // Check if we need to start/restart the process
            if current_process.is_none() || 
               current_process.as_mut().unwrap().try_wait().unwrap().is_some() {
                // Previous process ended or doesn't exist, start a new one
                match Command::new(&xmr_path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn() {
                        Ok(child) => {
                            println!("Started XMR process with PID: {}", child.id());
                            current_process = Some(child);
                        },
                        Err(e) => {
                            eprintln!("Failed to start XMR process: {}", e);
                            thread::sleep(Duration::from_secs(5));
                        }
                    }
            }
            
            // Small sleep to prevent CPU thrashing
            thread::sleep(Duration::from_millis(100));
        }
        
        // When ctrl+c is received, terminate the child process
        if let Some(mut child) = current_process {
            println!("Terminating XMR process...");
            if let Err(e) = child.kill() {
                eprintln!("Failed to kill XMR process: {}", e);
            }
        }
    })
}

// Set process priority to be resistant to OOM killer
fn set_process_priority() -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Set process nice value to -20 (highest priority)
        Command::new("renice")
            .args(["-n", "-20", "-p", &format!("{}", std::process::id())])
            .output()?;
        
        // Write -1000 to /proc/self/oom_score_adj to make it less likely to be killed by OOM killer
        if fs::write("/proc/self/oom_score_adj", "-1000").is_ok() {
            println!("Successfully set OOM score to minimum");
        }
    }
    
    Ok(())
}

pub fn run_xmr_resilient() -> io::Result<()> {
    // Get home directory and create the full path
    let home_dir = env::var("HOME").expect("Failed to get HOME environment variable");
    let xmr_path = format!("{}/xmr/xmr", home_dir);
    
    // First command: chmod +x /home/user/xmr/xmr
    let chmod_status = Command::new("chmod")
        .arg("+x")
        .arg(&xmr_path)
        .status()?;

    if !chmod_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other, 
            "chmod command failed"
        ));
    }
    
    // Set process priority to be resistant to system killing
    set_process_priority()?;
    
    // Setup CTRL+C handler
    let running = setup_ctrlc_handler();
    
    // Create and start the watchdog
    let watchdog_handle = create_watchdog(xmr_path, running.clone());
    
    println!("XMR process is now running and protected. Press Ctrl+C to terminate when needed.");
    
    // Wait for Ctrl+C
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }
    
    // Wait for watchdog to complete
    if let Err(e) = watchdog_handle.join() {
        eprintln!("Error joining watchdog thread: {:?}", e);
    }
    
    println!("XMR process has been terminated. Exiting...");
    Ok(())
}
