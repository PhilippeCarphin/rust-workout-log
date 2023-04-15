# Rust workout log

This is a workout tracking application that I made to help workout regularly.

I use Duolingo and it has the concept of a "streak" which once your streak
reaches about 5, the idea of missing a day and having it come back down to zero
becomes unthinkable.

The reasons for writing my own app which is pretty crappy compared to other
actual apps is to use my passion for programming to get me more excited about
working out.

# Data

A workout is a vector of exercises, an exercise is a vector of sets, and a set
is a weight and a number of reps.  This fails to represent things like timed
exercises like doing a plank and other types of exercises but covers most
gym style exercises.

As an example, a shoulder workout might be
- Overhead press: 15x10, 20x8, 25x3
- Lateral raise: 10x8, 10x8, 10x8.

This is represented by the following structs:
```rust
pub struct WorkoutHistory {
    workouts: Vec<Workout>,
    ongoing_workout: Option<Workout>
}

pub struct Workout {
    info: WorkoutInfo,
    exercises: Vec<Exercise>
}

pub struct Exercise {
    info: ExerciseInfo,
    sets: Vec<ExerciseSet>
}

pub struct ExerciseSet {
    weight: f64,
    reps: u8,
}
```
## The info types
I made a different struct for what could be called the "metatdata" of the workout
and exercise structs but I'm not seeing any benefit to doing that so I might get
rid of those info types and move their fileds into the Workout and Exercise types.

So this
```rust
pub struct ExerciseInfo {
    name: String,
    group: String,
}
pub struct Exercise {
    info: ExerciseInfo,
    sets: Vec<ExerciseSet>
}
```
would become
```rust
pub struct Exercise {
    name: String,
    group: String,
    sets: Vec<ExerciseSet>
}
```
since all that having separate types seems to do is to force me to write `exc.info.name`
to get the name of an exercise instead of `exc.name`.

## The `ongoing_workout` and the current `Exercise`

The idea of this program is that I would have it running in a REPL as I'm doing
a workout and type stuff like
```
begin-workout shoulders
begin-exercise overhead_press
enter-set 10.0 12
enter-set 15.0 10
enter-set 20.0 8
begin-exercise lateral_raise
...
end-workout
```

I wanted the data structures to reflect that there was an ongoing workout so
that if I stop the program during a workout, I can start it up and it will
remember that there was an ongoing workout.

I could very well have treated the last workout of the Worktout of 
`WorkoutHistory.workouts` as the ongoing workout and that would have worked but
it seemed to be a good opportunity to learn about Rust's `Option<T>`.

It indeed was a valuable learning experience, especially when coding the logic
to end the workout and move the workout contained in the `Option` into the 
vector of workouts.

I find it inconsistent that I'm not doing the same thing with the exercises
since `Workout::enter_set(&self, weight: f64, reps: u8)` gets called when
`"enter-set <w> <r>"` is typed in the REPL simply appends a set to the last
exercise in the Exercise vector of the workout.

The `WorkoutHistory::end_workout()` function took me a long time to figure out
because I really did not understand the ownership stuff at the time.

There is really not much of a  need for an `ongoing_workout`.  The last workout
could simply be considered ongoing until I do `"begin-workout" which would append
a new fresh workout to the vector of workouts and this workout would receive
the exercises and sets that I would subsequently enter.

For concistency, I think the current exercise should be treated like the
ongoing workout.
