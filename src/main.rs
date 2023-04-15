use crate::types::*;
mod types;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    let args : Vec<String> = std::env::args().collect();
    println!("Loading workout history ...");
    let mut wh = get_workout_data()?;
    println!("Workout history loaded.");
    //wh.streak(None)?;

    if args.len() == 1 {
        println!("Welcome to Phil's workout application 1.0.0");
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

