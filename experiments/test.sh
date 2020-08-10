#!/usr/bin/bash
cargo run "$1" --fixed-camera --max-iteration 100 --orbit-speed 0 --fullscreen \
--camera-x 15 --camera-z 0.0 --camera-height 30
