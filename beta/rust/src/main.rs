use std::env;

// Import the initialize function from init.rs
mod init;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let command = &args[1];
        
        match command.as_str() {
            "init" => {
                println!("Starting XMR initialization...");
                
                match init::initialize() {
                    Ok(()) => println!("Initialization completed successfully."),
                    Err(e) => eprintln!("Error during initialization: {}", e),
                }
            },
            _ => {
                println!("Unknown command: {}", command);
                println!("Available commands:");
                println!("  ./main init - Initialize XMR");
            }
        }
    } else {
        println!("Hello, world!");
        println!("Available commands:");
        println!("  ./main init - Initialize XMR");
    }
}
