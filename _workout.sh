#!/bin/bash

_workout(){

    local cur prev words cword
    _init_completion || return

    local commands="
        print
        begin-workout
        end-workout
        begin-exercise
        enter-set
        streak
        streak-status
        least-recent
        kg
        resume-workout
        csv
        help
    "

    if (( cword == 1 )) ; then
        COMPREPLY=($(compgen -W "${commands}" -- ${cur}))
    else
        case ${prev} in
            streak) return ;;
            streak-status) return ;;
            enter-set) return ;;
            begin-exercise) _workout_begin_exercise ;;
            begin-workout) _workout_begin_workout ;;
            print) return ;;
            least-recent) return ;;
            kg) return ;;
            resume-workout) return ;;
            csv) return ;;
            help) return ;;
        esac
    fi
}
alias w=workout
complete -F _workout workout w
