pub mod ramp;
use crate::ramp::install::install;

use std::io;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Print arguments for debugging
    println!("Arguments: {:?}", args);
    
    // Check for the -install argument, this flow requires sudo priveleges
    if args.contains(&"-install".to_string()) {
        println!("Running install with elevated privileges...");
        //initial install
        install()?;
        //TODO move the binary from the .dmg or the .deb after install is finished
        //TODO terminate the session
        //TODO can we start an external script with a timer here to relaunch ramp gui after closing initial install client?
    }else{
        #[cfg(not(target_arch="wasm32"))]
        { 
            main::maverick_main() 
        }
    }
    Ok(())
}