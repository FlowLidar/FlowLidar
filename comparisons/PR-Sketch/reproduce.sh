#!/usr/bin/bash
dir=./comparisons/PR-Sketch
make -C $dir clean 1>/dev/null 2>/dev/null
make -C $dir all 1>/dev/null 2>/dev/null
$dir/reproduce.py
