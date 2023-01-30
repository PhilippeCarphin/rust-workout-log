use serde::{Serialize, Deserialize};
use std::fs::File;
use rustyline;
use rustyline::Editor;
use dirs;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutHistory {
    workouts: Vec<Workout>,
    ongoing_workout: Option<Workout>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkoutInfo {
    date: String, // TODO Use actual datetime structure
    main_group: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workout {
    info: WorkoutInfo,
    exercises: Vec<Exercise>
}

// I don't think this struct is useful.  Its methods could probably become
// methods of WorkoutHistory
pub struct WorkoutManager {
    wh: WorkoutHistory,
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

impl WorkoutManager {
    fn handle_command(&mut self, line: String) {
        let words : Vec<&str> = line.split_whitespace().collect();
        if words.len() == 0 {
            return;
        }
        let command = words[0];
        let nargs = words.len() - 1;

        match &mut self.wh.ongoing_workout {
            Some(w) => {
                // wh.handle_command_ongoing()
                match command {
                    "enter-set" => {
                        if nargs < 2 {
                            println!("Not enough arguments for command '{}'", command);
                            return
                        }
                        // COnsider using words.get(i) instead, it returns
                        // an option with the thing if the index is in bounds
                        // and None if out of bounds.
                        let weight = words[1].parse::<f64>().unwrap();
                        let reps = words[2].parse::<u8>().unwrap();
                        w.enter_set(weight, reps);
                    },
                    "begin-exercise" => {
                        // Maybe have an "ongoing_exercise"
                        if nargs < 1 {
                            println!("ERROR: A name is required");
                            return
                        }
                        w.begin_exercise(String::from(words[1]))
                    },
                    "end-workout" => {
                        self.end_workout();
                    }
                    "begin-workout" => {
                        println!("Not implemented: {}", command);
                    }
                    _ => println!("Unknown command")
                }
            },
            None => {
                match command {
                    "begin-workout" => self.wh.ongoing_workout = Some(Workout {
                        info: WorkoutInfo {
                            date: String::from("today"),
                            main_group: String::from("shoulders")
                        },
                        exercises : Vec::<Exercise>::new()
                    }),
                    _ => {println!("ERROR, no ongoing workout.  The only valid command in this context is 'begin-workout'")}
                }
            }

        }
    }
    fn end_workout(&mut self){
        self.wh.end_workout();
    }
}


fn generate_sample_workout_history() -> WorkoutHistory {
    let mut wh = WorkoutHistory {
        workouts : Vec::<Workout>::new(),
        ongoing_workout: None
    };
    let mut today = Workout {
        info: WorkoutInfo { date: String::from("today"), main_group: String::from("shoulders") },
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
/*
 * new-workout
 */
pub fn repl(wh: WorkoutHistory) -> Result<(), rustyline::error::ReadlineError> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No history");
    }
    let mut wm = WorkoutManager {
        wh,
    };
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                wm.handle_command(line);
                println!("Workout : {:#?}", wm.wh.ongoing_workout);
            },
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
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
    // println!("Workout : {:#?}", wm.wh);
    match &wm.wh.ongoing_workout {
        Some(w) => {print_workout(&w);}
        None => {println!("No ongoing workout");}
    }
    print_workout_history(&wm.wh);

    let result = ::serde_json::to_writer_pretty(&File::create("/Users/pcarphin/.workout_data.json")?, &wm.wh);
    if result.is_err() {
        println!("Error saving to json");
    }

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
    println!("======= Complete history =======");
    for w in &wh.workouts {
        print_workout(w);
    }
}

fn get_workout_filename() -> core::result::Result<std::path::PathBuf, &'static str> {
    if let Some(d) = dirs::home_dir(){
        Ok(d.join(".workout_data.json"))
    } else {
        Err("Error")
    }
}

fn get_workout_file() -> Result<File, &'static str> {
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

pub fn get_workout_data() -> Result<WorkoutHistory, Box<dyn Error>> {
    let f = get_workout_file()?;
    let d = ::serde_json::from_reader(f)?;
    Ok(d)
}

pub fn test_workout_stuff(){
    let wh = generate_sample_workout_history();
    let mut today = Workout {
        info: WorkoutInfo { date: String::from("today"), main_group: String::from("shoulders") },
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
