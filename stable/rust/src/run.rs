use std::process::Command;
use std::env;
use std::io;

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
