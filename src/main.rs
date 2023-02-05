use crate::types::*;
mod types;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    let args : Vec<String> = std::env::args().collect();
    let mut wh = get_workout_data()?;
    wh.streak(None)?;

    if args.len() == 1 {
        repl(&mut wh)?;
    } else {
        wh.handle_command(&args[1..])?;
        print_workout_history(&wh);
    }
    Ok(())
}

