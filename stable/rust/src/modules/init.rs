use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn initialize() -> Result<(), String> {
    // Get the home directory path
    let home_dir = match env::var("HOME") {
        Ok(path) => path,
        Err(_) => return Err("Could not determine home directory".to_string()),
    };
    
    println!("Home directory: {}", home_dir);
    
    // Check if the XMR folder exists
    let xmr_path = Path::new(&home_dir).join("xmr");
    
    if xmr_path.exists() && xmr_path.is_dir() {
        println!("XMR folder already exists at {}", xmr_path.display());
        return Ok(());
    }
    
    // XMR folder doesn't exist, so download the zip file
    println!("XMR folder not found. Downloading XMR zip file...");
    
    let xmr_zip_url = "https://github.com/cazzano/Minning/releases/download/minning/xmr.zip";
    let zip_path = PathBuf::from(&home_dir).join("xmr.zip");
    
    // Download the zip file using wget
    let wget_status = Command::new("wget")
        .arg("-O")
        .arg(&zip_path)
        .arg(xmr_zip_url)
        .status()
        .map_err(|e| format!("Failed to execute wget: {}", e))?;
    
    if !wget_status.success() {
        return Err(format!("wget failed with exit code: {}", wget_status));
    }
    
    println!("Download completed. Extracting zip file...");
    
    // Unzip the file
    let unzip_status = Command::new("unzip")
        .arg("-o") // Overwrite files without prompting
        .arg(&zip_path)
        .arg("-d")
        .arg(&home_dir)
        .status()
        .map_err(|e| format!("Failed to execute unzip: {}", e))?;
    
    if !unzip_status.success() {
        return Err(format!("unzip failed with exit code: {}", unzip_status));
    }
    
    println!("Extraction completed successfully.");
    
    // Verify the XMR folder now exists
    if xmr_path.exists() && xmr_path.is_dir() {
        println!("XMR folder successfully created at {}", xmr_path.display());
        
        // Optionally, remove the zip file to clean up
        if let Err(e) = fs::remove_file(&zip_path) {
            println!("Warning: Could not remove zip file: {}", e);
        }
        
        Ok(())
    } else {
        Err("XMR folder was not created properly after extraction".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initialize() {
        // This is just a placeholder test
        // In a real scenario, you would mock the filesystem and commands
        // to test the functionality without actual changes
    }
}

fn main() {
    println!("Starting XMR initialization...");
    
    match initialize() {
        Ok(()) => println!("Initialization completed successfully."),
        Err(e) => eprintln!("Error during initialization: {}", e),
    }
}
