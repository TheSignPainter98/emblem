#!/usr/bin/bats

load common.bash

@test '.foreach executes correctly' {
	skip
	run $em << EOF
.foreach{d}{0 1 2 3}:
	.echo: !d
EOF
	assert_exit_pass
	[[ ${#lines[@]} -eq 4 ]]
	local sanitised_output=$(echo $output)
	[[ "$sanitised_output" == "0 1 2 3" ]]
}

@test '.foreach executes correctly when nested' {
	skip
	run $em << EOF
.foreach{d}{0 1}:
	.foreach{e}{0 1}:
		.echo: !d !e
EOF
	assert_exit_pass
	[[ ${#lines[@]} -eq 4 ]]
	local sanitised_output=$(echo $output)
	[[ "$sanitised_output" == "0 0 0 1 1 0 1 1" ]]
}

@test '.foreach returns correctly' {
	skip
	run $em << EOF
.foreach{d}{0 1}:
	!d
EOF
	assert_exit_pass
	[[ ${#lines[@]} -eq 2 ]]
	local sanitised_output=$(echo $output)
	[[ "$sanitised_output" == "0 1" ]]
}

@test '.foreach returns correctly when nested' {
	skip
	run $em << EOF
.foreach{d}{0 1}:
	.foreach{e}{0 1}:
		!d !e
EOF
	assert_exit_pass
	[[ ${#lines[@]} -eq 1 ]]
	[[ "$output" == "0 0 0 1 1 0 1 1" ]]
}
