use crate::types::*;
mod types;
use std::error::Error;
// use sscanf;
use chrono::TimeZone;
use chrono;

fn main() -> Result<(), Box<dyn Error>> {

    let d = chrono::Local::now();
    if let Ok(sd) = streak_date(d) {
        println!("streak_date for {} is {}", d, sd);
    }

    if let Some(d2) = chrono::Local.with_ymd_and_hms(2023,02,03,02,30,55).single() {
        if let Ok(sd) = streak_date(d2) {
            println!("streak_date for {} is {}", d2, sd);

        }
    }

    // if true {
    //     return Ok(());
    // }
    let args : Vec<String> = std::env::args().collect();
    let mut wh = get_workout_data()?;
    println!("Testing streak");
    wh.streak(streak_start_date_test()?)?;

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
    Ok(())
}

