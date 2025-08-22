#!/usr/bin/env bash

# ANSI color codes
GREEN='\033[1;32m'
CYAN='\033[1;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

ITER="$1"
FILE="$2"

echo -e "${CYAN}Benchmark iter=${ITER} file=${FILE}${NC}"

echo
echo -e "${BOLD}Running pretty:${NC}"
TIME_OUTPUT=$( (time ./target/release/pretty --iter "${ITER}" ${FILE} > /dev/null) 2>&1 )
REAL=$(echo "${TIME_OUTPUT}" | grep real | awk '{print $2}')
USER=$(echo "${TIME_OUTPUT}" | grep user | awk '{print $2}')
SYS=$(echo "${TIME_OUTPUT}" | grep sys | awk '{print $2}')
echo -e "  user:${USER} system: ${SYS} ${GREEN}total: ${REAL}${NC}"


echo
echo -e "${BOLD}Running pretty --no-color:${NC}"
TIME_OUTPUT=$( (time ./target/release/pretty --no-color --iter "${ITER}" ${FILE} > /dev/null) 2>&1 )
REAL=$(echo "${TIME_OUTPUT}" | grep real | awk '{print $2}')
USER=$(echo "${TIME_OUTPUT}" | grep user | awk '{print $2}')
SYS=$(echo "${TIME_OUTPUT}" | grep sys | awk '{print $2}')
echo -e "  user:${USER} system: ${SYS} ${GREEN}total: ${REAL}${NC}"

echo
echo -e "${BOLD}Running pretty --serde:${NC}"
TIME_OUTPUT=$( (time ./target/release/pretty --serde --iter "${ITER}" ${FILE} > /dev/null) 2>&1 )
REAL=$(echo "${TIME_OUTPUT}" | grep real | awk '{print $2}')
USER=$(echo "${TIME_OUTPUT}" | grep user | awk '{print $2}')
SYS=$(echo "${TIME_OUTPUT}" | grep sys | awk '{print $2}')
echo -e "  user:${USER} system: ${SYS} ${GREEN}total: ${REAL}${NC}"
