use crate::types::*;
mod types;
use std::error::Error;
// use sscanf;
use chrono::TimeZone;
use chrono;

fn main() -> Result<(), Box<dyn Error>> {

    let testing = std::env::var("RUST_WORKOUT_LOG_TESTING").is_ok();
    if testing {
        // TODO: Stop being a little bitch and do actual tests
        // - Adapt get_workout_data so that it works a test file instead of
        //   the real file
        // - Move this testing stuff into test files
        // - Adapt everyhing about the REPL so that I don't I can write
        //   automatic tests that will be as close as possible to me testing
        //   the REPL by running it and typing commands and confirming that
        //   the output is what I expect.
        println!("\x1b[1;33mTESTING MODE\x1b[0m");
        let d = chrono::Local::now();
        if let Ok(sd) = streak_date(d) {
            println!("streak_date for {} is {}", d, sd);
        }
        if let Some(d2) = chrono::Local.with_ymd_and_hms(2023,02,03,02,30,55).single() {
            if let Ok(sd) = streak_date(d2) {
                println!("streak_date for {} is {}", d2, sd);
            }
        }

        let args : Vec<String> = std::env::args().collect();
        let mut wh = get_workout_data()?;
        println!("Testing streak");
        wh.streak(None)?;

        if args.len() == 1 {
            repl(&mut wh)?;
        } else {
            wh.handle_command(&args[1..])?;
            print_workout_history(&wh);
        }

        wh.save()?;


        if false {
            // Quell unused function warning
            test_workout_stuff();
        }
    } else {
        let args : Vec<String> = std::env::args().collect();
        let mut wh = get_workout_data()?;
        wh.streak(None)?;

        if args.len() == 1 {
            repl(&mut wh)?;
        } else {
            wh.handle_command(&args[1..])?;
            print_workout_history(&wh);
        }
    }
    Ok(())
}

