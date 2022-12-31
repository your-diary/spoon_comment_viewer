#!/usr/bin/env bash

function f() {
    while true; do
        echo
        echo "--------------- $(date) ---------------"
        cargo run --release
        sleep 5
    done
}

f 2>&1 | tee -a './log.txt'

