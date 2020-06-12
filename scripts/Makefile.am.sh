#!/bin/bash

srcs=$(echo ./src/argp.c ./src/argp.h $(find -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$') | sed 's/\//\\\//g')

echo $srcs
sed "s/<SRC_FILES>/$srcs/" < /dev/stdin > /dev/stdout
