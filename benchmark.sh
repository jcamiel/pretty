#!/usr/bin/env zsh

echo "pretty:"
time cat 5mb.json | ./target/release/pretty --iter 500 > /dev/null

echo ""
echo "pretty --no-color:"
time cat 5mb.json | ./target/release/pretty --iter 500 --no-color > /dev/null

echo ""
echo "pretty --serde:"
time cat 5mb.json | ./target/release/pretty --iter 500 --serde > /dev/null
