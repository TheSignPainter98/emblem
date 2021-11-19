#!/usr/bin/bats

load common.bash

@test 'syntax error lines are reported' {
	run --output=stderr -- $em << EOF
.asdf:
EOF
	assert_exit_fail
	[[ "${lines[0]}" =~ ^\(stdin\):1 ]]
}

@test 'indent at EOF accepted' {
	run $em << EOF
.adsf:
	hfdjska
EOF
	assert_exit_pass
}

@test 'multi indent at EOF accepted' {
	run $em << EOF
.asdf:
	.fdsa:
		hjfdklsahfj
EOF
}

@test 'trailing indent at EOF rejected' {
	run $em << EOF
.asdf:
EOF
	assert_exit_fail
}
