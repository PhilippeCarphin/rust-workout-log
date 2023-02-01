use crate::types::*;
mod types;
use std::error::Error;
// use sscanf;

fn main() -> Result<(), Box<dyn Error>> {


    let args : Vec<String> = std::env::args().collect();
    let mut wh = get_workout_data()?;
    if args.len() == 1 {
        repl(&mut wh)?;
    } else {
        wh.handle_command(&args[1..]);
        print_workout_history(&wh);
    }

    wh.save()?;


    if false {
        // Quell unused function warning
        test_workout_stuff();
    }
    Ok(())
}

