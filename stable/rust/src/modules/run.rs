use std::process::Command;
use std::env;
use std::path::PathBuf;

fn main() {
    // Get home directory and create the full path
    let home_dir = env::var("HOME").expect("Failed to get HOME environment variable");
    let xmr_path = format!("{}/xmr/xmr", home_dir);
    
    // First command: chmod +x /home/user/xmr/xmr
    let chmod_status = Command::new("chmod")
        .arg("+x")
        .arg(&xmr_path)
        .status()
        .expect("Failed to execute chmod command");

    if !chmod_status.success() {
        eprintln!("Error: chmod command failed");
        return;
    }

    // Second command: execute /home/user/xmr/xmr
    let run_status = Command::new(&xmr_path)
        .status()
        .expect("Failed to execute the xmr command");

    if !run_status.success() {
        eprintln!("Error: xmr execution failed");
    }
}
