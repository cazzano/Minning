use std::env;

// Import the initialize function from init.rs
mod init;
// Import the run module
mod run;

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

            "run" => {
                println!("Running XMR...");
                
                match run::run_xmr() {
                    Ok(_) => println!("XMR executed successfully."),
                    Err(e) => {
                        eprintln!("Error running XMR: {}", e);
                        std::process::exit(1);
                    }
                }
            },
            _ => {
                println!("Unknown command: {}", command);
                println!("Available commands:");
                println!("  ./main init - Initialize XMR");
                println!("  ./main run - Run XMR");
            }
        }
    } else {
        println!("Hello, world!");
        println!("Available commands:");
        println!("  ./main init - Initialize XMR");
        println!("  ./main run - Run XMR");
    }
}
