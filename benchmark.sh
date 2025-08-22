#!/usr/bin/env bash

# ANSI color codes
GREEN='\033[1;32m'
CYAN='\033[1;36m'
NC='\033[0m' # No Color

ITER="$1"

echo -e "${CYAN}Running pretty (iter=${ITER}):${NC}"
TIME_OUTPUT=$( (time ./target/release/pretty --iter "${ITER}" 5mb.json > /dev/null) 2>&1 )
REAL=$(echo "${TIME_OUTPUT}" | grep real | awk '{print $2}')
USER=$(echo "${TIME_OUTPUT}" | grep user | awk '{print $2}')
SYS=$(echo "${TIME_OUTPUT}" | grep sys | awk '{print $2}')
echo -e "  user:${USER} system: ${SYS} ${GREEN}total: ${REAL}${NC}"


echo
echo -e "${CYAN}Running pretty --no-color (iter=${ITER}):${NC}"
TIME_OUTPUT=$( (time ./target/release/pretty --no-color --iter "${ITER}" 5mb.json > /dev/null) 2>&1 )
REAL=$(echo "${TIME_OUTPUT}" | grep real | awk '{print $2}')
USER=$(echo "${TIME_OUTPUT}" | grep user | awk '{print $2}')
SYS=$(echo "${TIME_OUTPUT}" | grep sys | awk '{print $2}')
echo -e "  user:${USER} system: ${SYS} ${GREEN}total: ${REAL}${NC}"

echo
echo -e "${CYAN}Running pretty --serde (iter=${ITER}):${NC}"
TIME_OUTPUT=$( (time ./target/release/pretty --serde --iter "${ITER}" 5mb.json > /dev/null) 2>&1 )
REAL=$(echo "${TIME_OUTPUT}" | grep real | awk '{print $2}')
USER=$(echo "${TIME_OUTPUT}" | grep user | awk '{print $2}')
SYS=$(echo "${TIME_OUTPUT}" | grep sys | awk '{print $2}')
echo -e "  user:${USER} system: ${SYS} ${GREEN}total: ${REAL}${NC}"
