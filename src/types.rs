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
    ongoing_workout: Option<Workout>,
    #[serde(default = "WorkoutHistory::default_value")]
    missing: String
}
impl WorkoutHistory {
    fn default_value() -> String {
        "DEFAULT VALUE".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutInfo {
    // date: String, // TODO Use actual datetime structure
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
// pub fn streak_date_1(dt: chrono::Local)  {
//     let d = dt.date_naive();
//     println!("Date {:?} becomes naive date {:?}", dt,d);
// 
// }

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
    fn end_workout(&mut self){
        if let Some(cw) = self.ongoing_workout.take() {
            self.workouts.push(cw);
        } else {
            println!("WARNING: WorkoutHistory::end_workout called with no ongoing workout");
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
}

impl Workout {
    fn begin_exercise(&mut self, name: String) {
        self.exercises.push(
            Exercise{
                info: ExerciseInfo{name, group: String::from("X")},
                sets: Vec::<ExerciseSet>::new()
            }
        );
    }
    fn enter_set(&mut self, weight: f64, reps: u8) {
        let result = self.exercises.last_mut();
        match result {
            Some(cur) => {
                let set = ExerciseSet{ weight, reps};
                println!("Adding set {:?} to exercise {:?}", set, cur.info);
                cur.sets.push(set);
            }
            None => println!("No current exercise")
        }
    }
}

impl WorkoutHistory {
    pub fn handle_command(&mut self, argv: &[String]) -> Result<String, Box<dyn Error>>{
        if argv.len() == 0 {
            return Err("Function requires at least a command".into());
        }
        let command = &argv[0];
        let nargs = argv.len() - 1;

        // TODO: This nesting is getting absolutely nuts, I have to fix it
        // somehow.
        match command.as_str() {
            "streak" =>{
                return match self.streak(None) {
                    Ok(n) => Ok(format!("Streak is {}", n)),
                    Err(e) => Err(e)
                }
            },
            "streak-status" => {
                println!("NOT IMPLEMENTED: {}", command)
            },
            "enter-set" => {
                if let Some(w) = &mut self.ongoing_workout {
                    if nargs < 2 {
                        return Err(format!("Not enough arguments for command {}", command).into());
                    }
                    // TODO: Get rid of this shameful index access
                    w.enter_set(argv[1].parse::<f64>()?, argv[2].parse::<u8>()?);
                } else {
                    println!("ERROR, command '{}' requires an ongoing workout", command);
                }
            },
            _ => match &mut self.ongoing_workout {
                Some(w) => {
                    // wh.handle_command_ongoing()
                    match command.as_str() {
                        "begin-exercise" => {
                            // Maybe have an "ongoing_exercise"
                            if nargs < 1 {
                                return Err("A name is required".into());
                            }
                            w.begin_exercise(String::from(&argv[1]))
                        },
                        "end-workout" => {
                            self.end_workout();
                        }
                        "begin-workout" => {
                            return Err(format!("{}: There is already an ongoing workout.  Maybe run 'end-workout'", command).into());
                        }
                        _ => {
                            return Err(format!("Unknown command {}", command).into());
                        }
                    }
                },
                None => {
                    match command.as_str() {
                        "begin-workout" => {
                            if nargs < 1 {
                                return Err(format!("Command {} requires a muscle group as an argument", command).into());
                            }
                            let d = chrono::Local::now();
                            self.ongoing_workout = Some(Workout {
                                info: WorkoutInfo {
                                    date: d,
                                    main_group: String::from(&argv[1])
                                },
                                exercises : Vec::<Exercise>::new()
                            })
                        },
                        _ => {
                            return Err(format!("no ongoing workout.  The only valid command in this context is 'begin-workout'").into());
                        }
                    }
                }
            }
        }
        Ok("Done".into())
    }
    pub fn save(&self) -> Result<(),Box<dyn Error>> {
        if let Ok(file) = create_workout_file() {
            ::serde_json::to_writer_pretty(file, self)?;
        }
        Ok(())
    }
}


fn generate_sample_workout_history() -> WorkoutHistory {
    let mut wh = WorkoutHistory {
        workouts : Vec::<Workout>::new(),
        ongoing_workout: None,
        missing: "MISSING VALUE".to_string()
    };
    let mut today = Workout {
        info: WorkoutInfo { date: chrono::Local::now(), main_group: String::from("shoulders") },
        exercises : Vec::<Exercise>::new()
    };
    let mut ohp: Exercise = Exercise {
        info: ExerciseInfo { name : String::from("overhead press"), group: String::from("shoulders")},
        sets: Vec::<ExerciseSet>::new()
    };
    ohp.sets.push(ExerciseSet{ weight: 0.0, reps: 12});
    ohp.sets.push(ExerciseSet{ weight: 5.0, reps: 12});
    ohp.sets.push(ExerciseSet{ weight: 5.0, reps: 12});
    ohp.sets.push(ExerciseSet{ weight: 5.0, reps: 12});
    // println!("{:?}", ohp);
    today.exercises.push(ohp);
    wh.workouts.push(today);
    // println!("{:?}", wh);

    return wh;
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

pub fn test_workout_stuff(){
    let wh = generate_sample_workout_history();
    let mut today = Workout {
        info: WorkoutInfo { date: chrono::Local::now(), main_group: String::from("shoulders") },
        exercises : Vec::<Exercise>::new()
    };

    today.begin_exercise(String::from("bench_press"));
    today.enter_set(10.0, 12);
    today.enter_set(15.0, 12);
    today.begin_exercise(String::from("squat"));
    today.enter_set(35.0, 15);
    println!("today's workout: {:#?}", today);

    if let Ok(f) = File::create("data.json") {
        let res = ::serde_json::to_writer_pretty(&f, &wh);
        if res.is_err() {
            println!("Could not save");
        }
    } else {
        return;
    }
}
