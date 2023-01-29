use serde::{Serialize, Deserialize};
use std::fs::File;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
// use sscanf;


#[derive(Serialize, Deserialize, Debug)]
struct WorkoutHistory {
    workouts: Vec<Workout>,
    ongoing_workout: Option<Workout>
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

#[derive(Serialize, Deserialize, Debug)]
struct WorkoutInfo {
    date: String, // TODO Use actual datetime structure
    main_group: String,
}

impl Workout {
    fn begin_exercise(&mut self, name: String) {
        self.exercises.push(Exercise{ info: ExerciseInfo{name, group: String::from("X")}, sets: Vec::<ExerciseSet>::new()});
    }
    fn enter_set(&mut self, weight: f64, reps: u8) {
        let result = self.exercises.last_mut();
        match result {
            Some(cur) => {
                let set = ExerciseSet{ weight: weight, reps: reps};
                println!("Adding set {:?} to exercise {:?}", set, cur.info);
                cur.sets.push(set);
            }
            None => println!("No current exercise")
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Workout {
    info: WorkoutInfo,
    exercises: Vec<Exercise>
}
// impl Workout {
//     fn clone(&self) -> Workout {
//         return Workout {
//             info: WorkoutInfo { date: self.info.date.clone(), main_group: self.info.date.clone() },
//             exercises: self.exercises.clone()
//         };
//     }
// }

struct WorkoutManager {
    wh: WorkoutHistory,
}
impl WorkoutManager {
    fn handle_command(&mut self, line: String) {
        match &mut self.wh.ongoing_workout {
            Some(w) => {
                println!("TODO Handle line");
                println!("line = '{}'", line);
                let words : Vec<&str> = line.split_whitespace().collect();
                if words.len() == 0 {
                    return;
                }
                let command = words[0];
                let nargs = words.len() - 1;
                match command {
                    "enter-set" => {
                        if nargs < 2 {
                            println!("Not enough arguments for command '{}'", command);
                            return
                        }
                        let weight = words[1].parse::<f64>().unwrap();
                        let reps = words[2].parse::<u8>().unwrap();
                        w.enter_set(weight, reps);
                    },
                    "begin-exercise" => {
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
            None => self.wh.ongoing_workout = Some(Workout {
                info: WorkoutInfo { date: String::from("today"), main_group: String::from("shoulders") },
                exercises : Vec::<Exercise>::new()
            })
        }
    }
    fn end_workout(&mut self){
        self.wh.end_workout();
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct ExerciseInfo {
    name: String,
    group: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExerciseSet {
    weight: f64,
    reps: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Exercise {
    info: ExerciseInfo,
    sets: Vec<ExerciseSet>
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
fn repl(wh: WorkoutHistory) -> Result<()> {
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
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
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

    let result = ::serde_json::to_writer_pretty(&File::create("replsave.json")?, &wm.wh);
    if result.is_err() {
        println!("Error saving to json");
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut wh = generate_sample_workout_history();

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

    ::serde_json::to_writer_pretty(&File::create("data.json")?, &wh)?;
    let res : WorkoutHistory = ::serde_json::from_reader(std::fs::File::open("data2.json")?)?;
    println!("{:?}", res);
    let repl_res = repl(wh);
    if repl_res.is_err() {
        println!("Error in REPL");
    }
    Ok(())
}
