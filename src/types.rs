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

impl WorkoutHistory {
    fn end_workout(&mut self){
        if let Some(cw) = self.ongoing_workout.take() {
            self.workouts.push(cw);
        } else {
            println!("WARNING: WorkoutHistory::end_workout called with no ongoing workout");
        }
    }
    fn streak(&self) -> i32 {
        let mut n = 0;
        let mut prev : Option<&Workout> = None;
        for w in self.workouts.iter().rev() {
            // TODO: Start streak analysis with current date
            // TODO: 
            match prev {
                Some(wp) => {
                    let dp = wp.info.date;
                    let d = w.info.date;
                    println!("Streak analysis: prev = {}, cur = {}", dp, d);
                    let diff = dp.signed_duration_since(d);
                    println!("signed duration: {}, num_days: {}, num_hours: {}", diff, diff.num_days(), diff.num_hours());
                    if diff.num_days() <= 1 {
                        n += 1;
                    } else {
                        break;
                    }
                }
                None => {
                    println!("First iteration, no previous workout");
                }
            }
            prev = Some(&w);
        }
        println!("The current streak is {n}");
        return n;
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
    pub fn handle_command(&mut self, argv: &[String]) {
        if argv.len() == 0 {
            return;
        }
        let command = &argv[0];
        let nargs = argv.len() - 1;

        // TODO: This nesting is getting absolutely nuts, I have to fix it
        // somehow.
        match command.as_str() {
            "streak" =>{
                self.streak();
                return;
            },
            "streak-status" => {
                println!("NOT IMPLEMENTED: {}", command)
            }
            _ => match &mut self.ongoing_workout {
                Some(w) => {
                    // wh.handle_command_ongoing()
                    match command.as_str() {
                        "enter-set" => {
                            if nargs < 2 {
                                println!("Not enough arguments for command '{}'", command);
                                return
                            }
                            // COnsider using words.get(i) instead, it returns
                            // an option with the thing if the index is in bounds
                            // and None if out of bounds.
                            let weight = argv[1].parse::<f64>().unwrap();
                            let reps = argv[2].parse::<u8>().unwrap();
                            w.enter_set(weight, reps);
                        },
                        "begin-exercise" => {
                            // Maybe have an "ongoing_exercise"
                            if nargs < 1 {
                                println!("ERROR: A name is required");
                                return
                            }
                            w.begin_exercise(String::from(&argv[1]))
                        },
                        "end-workout" => {
                            self.end_workout();
                        }
                        "begin-workout" => {
                            println!("{}: ERROR: There is already an ongoing workout.  Maybe run 'end-workout'", command);
                        }
                        _ => println!("Unknown command")
                    }
                },
                None => {
                    match command.as_str() {
                        "begin-workout" => {
                            if nargs < 1 {
                                println!("ERROR: Command {} requires a muscle group as an argument", command);
                                return
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
                        _ => {println!("ERROR, no ongoing workout.  The only valid command in this context is 'begin-workout'")}
                    }
                }
            }
        }
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
        ongoing_workout: None
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
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let argv = shell_words::split(line.as_str())?;
                wh.handle_command(&argv);
                println!("Workout : {:#?}", wh.ongoing_workout);
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
    rl.save_history("history.txt")?;
    // println!("Workout : {:#?}", wh);
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

fn get_workout_filename() -> core::result::Result<std::path::PathBuf, &'static str> {
    let mut path = std::path::PathBuf::new();
    path.push("workout_data.json");
    return Ok(path);
    // if let Some(d) = dirs::home_dir(){
    //     Ok(d.join(".workout_data.json"))
    // } else {
    //     Err("Could not get home directory")
    // }
}

/*
 * std::fs::File::open(name) opens a file in read only mode and
 * std::fs::File::create(name) opens a file in write only mode and truncates
 * the file.  I thought of having a get_workout_file(write: bool) but
 * then the function calls get_workout_file(true) and get_workout_file(false)
 * look kind of silly.  Now I have these two functions that are almost
 * identical which also looks silly but this way is clearer.
 */
fn open_workout_file() -> Result<File, &'static str> {
    match get_workout_filename() {
        Ok(filename) => {
            match std::fs::File::open(filename) {
                Ok(file) => {
                    Ok(file)
                },
                Err(_e) => {
                    return Err("Could not open workout file");
                }
            }
        },
        Err(e) => {
            Err(e)
        }
    }
}

fn create_workout_file() -> Result<File, &'static str> {
    match get_workout_filename() {
        Ok(filename) => {
            match std::fs::File::create(filename) {
                Ok(file) => {
                    Ok(file)
                },
                Err(_e) => {
                    return Err("Could not open workout file");
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
