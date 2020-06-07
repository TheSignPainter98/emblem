#!/bin/zsh

set -e

echo -e "# Change Log\n"

git log --pretty=format:%s\
	| grep -v '^Merge'\
	| grep -v '^Initial commit$'\
	| sed 's/^/- /'
