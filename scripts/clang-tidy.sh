#/bin/bash

read -r -d' ' check_regexes << EOF
bugprone-.*
cert-.*-c
clang-analyzer-.*
clang-diagnostic-.*
google-[^r].*
google-readability-[^b].*
openmp-.*
performance-.*
readability-[^b].*
EOF

check_regex=$(for l in ${check_regexes[@]}; do printf '^%s$\n' $l; done | tr '\n' '|' | sed 's/|$//')
checks=$(clang-tidy --checks='*' --list-checks | grep '^[ ]' | cut -d' ' -f5- | grep -P "$check_regex" | sort | tr '\n' ',' | sed 's/,$//')

m4 -PE - .clang-tidy.in > .clang-tidy << EOF
m4_define(S_CHECKS, \`$checks')m4_dnl
EOF
