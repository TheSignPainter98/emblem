#/bin/bash

read -r -d' ' check_regexes << EOF
bugprone-.*
cert-.*-c
clang-analyzer-.*
clang-diagnostic-.*
google-[^r].*
google-readability-[^b].*
modernize-*
openmp-.*
performance-.*
readability-[^b].*
EOF

read -r -d' ' warn_only_regexes << EOF
clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling
EOF

function mkregex()
{
	for r in $@; do printf '^%s$\n' $r; done | tr '\n' '|' | sed 's/|$//'
}

function mkclanglist()
{
	sort | tr '\n' ',' | sed 's/,$//'
}

check_regex=$(mkregex ${check_regexes[@]})
warn_regex=$(mkregex ${warn_only_regexes[@]})

supported_checks=($(clang-tidy --checks='*' --list-checks | grep '^[ ]' | cut -d' ' -f5-))

checks_to_do=$(for c in ${supported_checks[@]}; do echo $c; done | grep -P "$check_regex" | mkclanglist)
error_checks=$(for c in ${supported_checks[@]}; do echo $c; done | grep -vP "$warn_regex" | mkclanglist)

m4 -PE - .clang-tidy.in > .clang-tidy << EOF
m4_define(S_CHECKS, \`$checks_to_do')m4_dnl
m4_define(S_ERRORS, \`$error_checks')m4_dnl
EOF
