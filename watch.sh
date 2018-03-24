#!/bin/bash
# Builds and runs the project on every change.
while inotifywait -e close_write src/*.rs > /dev/null 2>/dev/null; do
  clear
  timeout 1s cargo run
done
