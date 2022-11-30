#!/bin/sh

# 1. run cargo tree
# 2. remove compatible duplicate libraries (as signified by the asterix)
# 3. count the lines
cargo tree | greo -ve '*' | wc -l
