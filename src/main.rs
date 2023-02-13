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
        match wh.handle_command(&args[1..]) {
            Ok(resp) => println!("{}", resp),
            Err(e) => return Err(e)
        };
        wh.save()?
    }
    Ok(())
}

