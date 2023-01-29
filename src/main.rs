use serde::{Serialize, Deserialize};
use std::fs::File;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use sscanf;


#[derive(Serialize, Deserialize, Debug)]
struct WorkoutHistory {
    workouts: Vec<Workout>,
    ongoing_workout: Option<Workout>
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkoutInfo {
    date: String, // TODO Use actual datetime structure
    main_group: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OngoingWorkout {
    workout: Workout
}

impl Workout {
    fn begin_exercise(&mut self, name: String) {
        self.exercises.push(Exercise{ info: ExerciseInfo{ name: name, group: String::from("X")}, sets: Vec::<ExerciseSet>::new()});
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


#[derive(Serialize, Deserialize, Debug)]
struct Workout {
    info: WorkoutInfo,
    exercises: Vec<Exercise>
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
fn repl(wh: WorkoutHistory) -> Result<()> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No history");
    }
    let mut workout = Workout {
        info: WorkoutInfo { date: String::from("today"), main_group: String::from("shoulders") },
        exercises : Vec::<Exercise>::new()
    };
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("TODO Handle line");
                let parsed = sscanf::sscanf!(line, "{} {}", String, String);
                let (command, args) = parsed.unwrap();
                match command.as_str() {
                    "enter-set" => {
                        let parsed = sscanf::sscanf!(args, "{} {}", f64, u8);
                        let (weight, reps) = parsed.unwrap();
                        workout.enter_set(weight, reps);
                    },
                    "begin-exercise" => {
                        let parsed = sscanf::sscanf!(args, "{}", String);
                        let name = parsed.unwrap();
                        workout.begin_exercise(name)
                    },
                    _ => println!("Unknown command")
                }
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

    Ok(())
}

fn main() -> std::io::Result<()> {
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
    

    ::serde_json::to_writer_pretty(&File::create("data.json")?, &wh)?;
    let res : WorkoutHistory = ::serde_json::from_reader(std::fs::File::open("data2.json")?)?;
    println!("{:?}", res);
    let repl_res = repl(wh);
    if repl_res.is_err() {
        println!("Error in REPL");
    }
    Ok(())
}
