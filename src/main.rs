use crate::types::*;
mod types;
use std::error::Error;
// use sscanf;

fn main() -> Result<(), Box<dyn Error>> {

    repl(get_workout_data()?)?;

    if false {
        // Quell unused function warning
        test_workout_stuff();
    }
    Ok(())
}

