use crate::types::*;
mod types;
use std::error::Error;
// use sscanf;

fn main() -> Result<(), Box<dyn Error>> {


    let args : Vec<String> = std::env::args().collect();
    println!("Command arguments : '{:#?}'", args);
    let mut wh = get_workout_data()?;
    if args.len() == 1 {
        repl(wh)?;
    } else {
        wh.handle_command(&args[1..]);
    }


    if false {
        // Quell unused function warning
        test_workout_stuff();
    }
    Ok(())
}

