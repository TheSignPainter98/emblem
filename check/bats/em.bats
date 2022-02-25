#!/usr/bin/bats

load common.bash

@test 'em reads from the pipe by default' {
	local text='Hello, world!'
	run $em << EOF
$text
EOF
	assert_exit_pass
	[[ "${lines[0]}" =~ ^${text}$ ]]
	[[ "${#lines[@]}" -eq 1 ]]
}

@test '.echo outputs text' {
	local text='Hello, world!'
	local out_file="$BATS_TEST_TMPDIR/file.txt"
	run $em -o $out_file << EOF
.echo: $text
EOF
	local output_contents=$(cat < $out_file)
	assert_exit_pass
	[[ "${lines[0]}" =~ ^${text}$ ]]
	[[ "$output_contents" == "" ]]
}

@test 'em reads from a given file' {
	local file_loc="$BATS_TEST_TMPDIR/file.em"
	local text='Hello, world! This is from a file!'
	cat > $file_loc << EOF
$text
EOF
	run $em $file_loc
	assert_exit_pass
	[[ "${lines[0]}" =~ ^${text}$ ]]
	[[ "${#lines[@]}" -eq 1 ]]
}
