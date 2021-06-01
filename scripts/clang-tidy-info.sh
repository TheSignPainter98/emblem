#!/bin/bash

(echo -n "["; \
for file in $(./scripts/lintable-srcs.sh); do
	[[ ! -f $file ]] && make file >&2
	awk -f scripts/linting-line-filter.awk -v file=$file < $file
done | sed 's/,$//';
echo -n "]") > clang-tidy-info.json
