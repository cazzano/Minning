use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::env;
use std::process::{self, Command};
use std::os::unix::fs::PermissionsExt;

/// Expands the tilde in a path to the user's home directory
fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = env::var("HOME").expect("Failed to get HOME environment variable");
        let path_without_tilde = &path[2..]; // Remove the ~/ from the path
        Path::new(&home).join(path_without_tilde)
    } else {
        Path::new(path).to_path_buf()
    }
}

/// Checks if the current process has root privileges
fn has_root_privileges() -> bool {
    // Get the effective user ID using a direct system call
    // This avoids needing the libc crate as a dependency
    let output = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to execute id command");
    
    let uid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .expect("Failed to parse UID");
    
    // Root has UID 0
    uid == 0
}

/// Re-executes the current program with sudo
fn rerun_with_sudo() -> io::Result<()> {
    // Get the path to the current executable
    let current_exe = env::current_exe()?;
    
    println!("This operation requires administrative privileges.");
    println!("Requesting sudo access...");
    
    // Execute sudo with the current program
    let status = Command::new("sudo")
        .arg(current_exe)
        .status()?;
    
    // Exit with the same status as the sudo command
    process::exit(status.code().unwrap_or(0));
}

/// Copies all .txt files from the source directory to the destination directory
fn copy_txt_files() -> io::Result<()> {
    let source_dir = expand_tilde("~/xmr/");
    let source_dir_display = source_dir.clone(); // Create a clone for display purposes
    let dest_dir = PathBuf::from("/usr/bin/");

    // Check if source directory exists
    if !source_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory {:?} not found", source_dir),
        ));
    }

    // Check if destination directory exists
    if !dest_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Destination directory {:?} not found", dest_dir),
        ));
    }

    // Check if we have write permission to the destination
    match fs::metadata(&dest_dir) {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            // Check if directory is writable by current user
            // This is a simplified check - actual permission checking is more complex
            if (permissions.mode() & 0o200) == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Destination directory is not writable. You need administrative privileges.",
                ));
            }
        },
        Err(e) => return Err(e),
    }

    println!("Copying .txt files from {:?} to {:?}", source_dir_display, dest_dir);

    // Count of files copied
    let mut copied_files = 0;

    // Iterate through the entries in the source directory
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Check if it's a file with .txt extension
        if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
            let file_name = path.file_name().unwrap();
            let dest_path = dest_dir.join(file_name);
            
            println!("Copying {:?} to {:?}", path, dest_path);
            fs::copy(&path, &dest_path)?;
            copied_files += 1;
        }
    }

    if copied_files > 0 {
        println!("File copy operation completed successfully. Copied {} files.", copied_files);
    } else {
        println!("No .txt files found in {:?} to copy.", source_dir_display);
    }
    
    Ok(())
}

fn main() {
    // Check if we're running with root privileges
    if !has_root_privileges() {
        // If not, re-run with sudo
        if let Err(e) = rerun_with_sudo() {
            eprintln!("Failed to obtain root privileges: {}", e);
            process::exit(1);
        }
        // The rerun_with_sudo function will exit the program, so if execution continues,
        // it means there was an error but we didn't detect it.
        eprintln!("Unexpected error when trying to obtain root privileges");
        process::exit(1);
    }

    println!("Starting file copy operation with administrative privileges...");
    
    // Run the copy operation and handle any errors
    match copy_txt_files() {
        Ok(_) => println!("Operation completed."),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
