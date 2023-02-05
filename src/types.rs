use serde::{Serialize, Deserialize};
use chrono;
use std::fs::File;
use rustyline;
use rustyline::Editor;
use dirs;
use std::error::Error;
use shell_words;

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutHistory {
    workouts: Vec<Workout>,
    ongoing_workout: Option<Workout>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutInfo {
    date: chrono::DateTime<chrono::Local>,
    main_group: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workout {
    info: WorkoutInfo,
    exercises: Vec<Exercise>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExerciseInfo {
    name: String,
    group: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExerciseSet {
    weight: f64,
    reps: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exercise {
    info: ExerciseInfo,
    sets: Vec<ExerciseSet>
}

pub fn cutoff_time() -> Result<chrono::NaiveTime, Box<dyn Error>> {
    return chrono::NaiveTime::from_hms_opt(3,3,3)
        .ok_or("Could not create cutoff time".into())
}

/*
 * A workout done on Tuesday before 3:30 AM counts as a workout done
 * Monday for the purposes of streak computations.
 */
pub fn streak_date(dt: chrono::DateTime<chrono::Local>) -> Result<chrono::NaiveDate, Box<dyn Error>> {
    if dt.time() < cutoff_time()? {
        return dt.date_naive().checked_sub_days(chrono::Days::new(1))
            .ok_or("Could not subtract one day when computing streak date".into())
    } else {
        Ok(dt.date_naive())
    }
}

pub fn tomorrow() -> Result<chrono::NaiveDate, Box<dyn Error>>{
    return streak_date(chrono::Local::now())?
        .checked_add_days(chrono::Days::new(1))
        .ok_or("Could not subtract one day from today".into());
}

impl WorkoutHistory {
    fn end_workout(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(cw) = self.ongoing_workout.take() {
            self.workouts.push(cw);
            Ok(())
        } else {
            Err("No ongoing workout".into())
        }
    }
    pub fn streak(&self, start_date: Option<chrono::NaiveDate>) -> Result<i32, Box<dyn Error>> {
        let mut n : i32 = 0;

        let mut prev = start_date.unwrap_or(tomorrow()?);
        for w in self.workouts.iter().rev() {
            let cur = streak_date(w.info.date)?;
            let diff = prev.signed_duration_since(cur);
            match diff.num_days() {
                i64::MIN..=-1 => {
                    // println!("ERROR: Workouts out of order in database");
                    return Err("Workouts are out of order, cannot reliably calculate streak".into());
                },
                0 => {
                    continue;
                },
                1 => {
                    n += 1;
                },
                2..=i64::MAX => {
                    // println!("Streak ends at {} since the previous workout is on {}", prev, cur);
                    return Ok(n);
                }
            }
            prev = cur;
        }
        Ok(n)
    }
    fn begin_workout(&mut self, main_group: String) -> Result<(), Box<dyn Error>> {
        let d = chrono::Local::now();
        self.ongoing_workout = Some(Workout {
            info: WorkoutInfo {
                date: d,
                main_group
            },
            exercises : Vec::<Exercise>::new()
        });
        Ok(())
    }

    fn enter_set_command(&mut self, argv: &[String] ) -> Result<String, Box<dyn Error>> {
        if let Some(w) = &mut self.ongoing_workout {
            if argv.len() < 2 {
                return Err("Not enough arguments".into());
            }
            // TODO: Get rid of this shameful index access
            w.enter_set(argv[0].parse::<f64>()?, argv[1].parse::<u8>()?)?;
            Ok("Set added".into())
        } else {
            Err("No ongoing workout".into())
        }
    }
    fn begin_exercise_command(&mut self, argv: &[String]) -> Result<String, Box<dyn Error>> {
        return match &mut self.ongoing_workout {
            Some(w) => {
                if argv.len() < 1 {
                    Err("A name is required".into())
                } else {
                    w.begin_exercise(String::from(&argv[0]))
                }
            },
            None => {
                Err("No ongoing workout".into())
            }
        }
    }
    fn end_workout_command(&mut self, _argv: &[String]) -> Result<String, Box<dyn Error>> {
        self.end_workout()?;
        Ok("Workout ended".into())
    }

    fn begin_workout_command(&mut self, argv: &[String]) -> Result<String, Box<dyn Error>> {
        if argv.len() < 1 {
            return Err("Missing muscle group argument".into());
        }
        self.begin_workout(argv[0].to_string())?;
        return Ok("Workout started".into())
    }
    fn streak_command(&self, _argv: &[String]) -> Result<String, Box<dyn Error>> {
        match self.streak(None) {
            Ok(n) => Ok(format!("Streak is {}", n)),
            Err(e) => Err(e)
        }
    }
}

impl Workout {
    fn begin_exercise(&mut self, name: String) -> Result<String, Box<dyn Error>> {
        self.exercises.push(
            Exercise{
                info: ExerciseInfo{name, group: String::from("X")},
                sets: Vec::<ExerciseSet>::new()
            }
        );
        return Ok("Exercise started".to_string())
    }
    fn enter_set(&mut self, weight: f64, reps: u8) -> Result<(), Box<dyn Error>> {
        let result = self.exercises.last_mut();
        match result {
            Some(cur) => {
                let set = ExerciseSet{ weight, reps};
                cur.sets.push(set);
                Ok(())
            }
            None => Err("No current exercise".into())
        }
    }
}

impl WorkoutHistory {
    pub fn handle_command(&mut self, argv: &[String]) -> Result<String, Box<dyn Error>>{
        if argv.len() == 0 {
            return Err("Function requires at least a command".into());
        }
        let command = &argv[0];
        let args = &argv[1..];

        let res = match command.as_str() {
            "streak" => self.streak_command(args),
            "streak-status" => Err(format!("{}: NOT IMPLEMENTED",command).into()),
            "enter-set" => self.enter_set_command(args),
            "begin-exercise" => self.begin_exercise_command(args),
            "end-workout" => self.end_workout_command(args),
            "begin-workout" => self.begin_workout_command(args),
            _ => return Err(format!("{}: no such command", command).into())
        };

        if let Err(e) = res {
            return Err(format!("{}: {}", command, e).into())
        } else {
            return res
        }
    }
    pub fn save(&self) -> Result<(),Box<dyn Error>> {
        if let Ok(file) = create_workout_file() {
            ::serde_json::to_writer_pretty(file, self)?;
        }
        Ok(())
    }
}

pub fn repl(wh: &mut WorkoutHistory) -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No history");
    }
    // TODO: Set beginning prompt based on workout history
    // Ongoing workout or not, current exercise, set number?
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line == "" {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                let argv = shell_words::split(line.as_str())?;
                if let Err(e) = wh.handle_command(&argv) {
                    println!("\x1b[1;31mERROR\x1b[0m: {}", e);
                }
                rl.save_history("history.txt")?;
                if wh.save().is_err() {
                    println!("\x1b[1;31ERROR\x1b[0m Failed to save file");
                }
                if let Some(ow) = &wh.ongoing_workout {
                    print_workout(ow);
                } else {
                    print_workout_history(&wh);
                }
            },
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
            },
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("ERROR {:?}", err);
                break
            }
        }
    }
    print_workout_history(&wh);

    Ok(())
}

pub fn print_workout(w : & Workout) {
    println!("{} workout done on {}", w.info.main_group, w.info.date);
    for e in &w.exercises {
        print!("    {}: ", e.info.name);
        for s in &e.sets {
            print!("{}x{}; ", s.weight, s.reps);
        }
        print!("\n");
    }
}

pub fn print_workout_history(wh: &WorkoutHistory) {
    match &wh.ongoing_workout {
        Some(w) => {
            println!("======= Ongoing workout =======");
            print_workout(&w);
        }
        None => {println!("No ongoing workout");}
    }
    println!("======= Complete history =======");
    for w in &wh.workouts {
        print_workout(w);
    }
}

fn get_workout_filename() -> core::result::Result<std::path::PathBuf, Box<dyn Error>> {
    let testing = std::env::var("RUST_WORKOUT_LOG_TESTING").is_ok();
    if testing {
        return Ok(std::path::PathBuf::from("workout_data.json"));
    } else {
        if let Some(d) = dirs::home_dir(){
            Ok(d.join(".workout_data.json"))
        } else {
            Err("Could not get home directory".into())
        }
    }
}

/*
 * std::fs::File::open(name) opens a file in read only mode and
 * std::fs::File::create(name) opens a file in write only mode and truncates
 * the file.  I thought of having a get_workout_file(write: bool) but
 * then the function calls get_workout_file(true) and get_workout_file(false)
 * look kind of silly.  Now I have these two functions that are almost
 * identical which also looks silly but this way is clearer.
 */
fn open_workout_file() -> Result<File, Box<dyn Error>> {
    match get_workout_filename() {
        Ok(filename) => {
            match std::fs::File::open(filename) {
                Ok(file) => {
                    Ok(file)
                },
                Err(_e) => {
                    return Err("Could not open workout file".into());
                }
            }
        },
        Err(e) => {
            Err(e)
        }
    }
}

fn create_workout_file() -> Result<File, Box<dyn Error>> {
    match get_workout_filename() {
        Ok(filename) => {
            match std::fs::File::create(filename) {
                Ok(file) => {
                    Ok(file)
                },
                Err(_e) => {
                    return Err("Could not open workout file".into());
                }
            }
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub fn get_workout_data() -> Result<WorkoutHistory, Box<dyn Error>> {
    let f = open_workout_file()?;
    let d = ::serde_json::from_reader(f)?;
    Ok(d)
}
