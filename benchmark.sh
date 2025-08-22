#!/usr/bin/env bash

ITER="$1"

echo "Running pretty (iter=$ITER):"
TIME_OUTPUT=$( (time ./target/release/pretty --iter "$ITER" 5mb.json > /dev/null) 2>&1 )
REAL=$(echo "$TIME_OUTPUT" | grep real | awk '{print $2}')
USER=$(echo "$TIME_OUTPUT" | grep user | awk '{print $2}')
SYS=$(echo "$TIME_OUTPUT" | grep sys | awk '{print $2}')
echo "  user:$USER system: $SYS total: $REAL"


echo
echo "Running pretty --no-color (iter=$ITER):"
TIME_OUTPUT=$( (time ./target/release/pretty --no-color --iter "$ITER" 5mb.json > /dev/null) 2>&1 )
REAL=$(echo "$TIME_OUTPUT" | grep real | awk '{print $2}')
USER=$(echo "$TIME_OUTPUT" | grep user | awk '{print $2}')
SYS=$(echo "$TIME_OUTPUT" | grep sys | awk '{print $2}')
echo "  user:$USER system: $SYS total: $REAL"

echo
echo "Running pretty --serde (iter=$ITER):"
TIME_OUTPUT=$( (time ./target/release/pretty --serde --iter "$ITER" 5mb.json > /dev/null) 2>&1 )
REAL=$(echo "$TIME_OUTPUT" | grep real | awk '{print $2}')
USER=$(echo "$TIME_OUTPUT" | grep user | awk '{print $2}')
SYS=$(echo "$TIME_OUTPUT" | grep sys | awk '{print $2}')
echo "  user:$USER system: $SYS total: $REAL"
