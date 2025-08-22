#!/usr/bin/env bash

NB_ITER="$1"

echo "Running pretty (iter=$NB_ITER):"
TIME_OUTPUT=$( (time cat 5mb.json | ./target/release/pretty --iter "$NB_ITER" > /dev/null) 2>&1 )
REAL=$(echo "$TIME_OUTPUT" | grep real | awk '{print $2}')
USER=$(echo "$TIME_OUTPUT" | grep user | awk '{print $2}')
SYS=$(echo "$TIME_OUTPUT" | grep sys | awk '{print $2}')
echo "  user:$USER system: $SYS total: $REAL"


echo
echo "Running pretty --no-color (iter=$NB_ITER):"
TIME_OUTPUT=$( (time cat 5mb.json | ./target/release/pretty --no-color --iter "$NB_ITER" > /dev/null) 2>&1 )
REAL=$(echo "$TIME_OUTPUT" | grep real | awk '{print $2}')
USER=$(echo "$TIME_OUTPUT" | grep user | awk '{print $2}')
SYS=$(echo "$TIME_OUTPUT" | grep sys | awk '{print $2}')
echo "  user:$USER system: $SYS total: $REAL"

echo
echo "Running pretty --serde (iter=$NB_ITER):"
TIME_OUTPUT=$( (time cat 5mb.json | ./target/release/pretty --serde --iter "$NB_ITER" > /dev/null) 2>&1 )
REAL=$(echo "$TIME_OUTPUT" | grep real | awk '{print $2}')
USER=$(echo "$TIME_OUTPUT" | grep user | awk '{print $2}')
SYS=$(echo "$TIME_OUTPUT" | grep sys | awk '{print $2}')
echo "  user:$USER system: $SYS total: $REAL"
