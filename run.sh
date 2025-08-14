#!/bin/bash

cat 5mb.json | target/release/pretty --iter 5000 > /dev/null
