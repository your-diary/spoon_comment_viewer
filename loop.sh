#!/usr/bin/env bash

function f() {
    echo
    echo "--------------- $(date) ---------------"
    while true; do
        cargo run --release
        sleep 5
    done
}

f 2>&1 | tee -a './log.txt'

