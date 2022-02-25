#!/usr/bin/bats

load common.bash

@test "--help outputs usage information" {
	run --output=stderr -- $em --help
	assert_exit_pass
	[[ "${lines[0]}" =~ ^Usage: ]]
}

@test "unrecognised parameters trigger usage info output" {
	run --output=stderr -- $em --unrecognised
	assert_exit_fail
	[[ "${lines[0]}" =~ '--unrecognised' ]]
	[[ "${lines[1]}" =~ ^Usage: ]]
}

@test '--version outputs version and license' {
	run $em --version
	assert_exit_pass
	[[ "${lines[0]}" =~ ^em ]]
	[[ "${lines[0]}" =~ [0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]* ]]
	[[ "${output,,}" =~ copyright ]]
	[[ "${output,,}" =~ license ]]
}

@test 'license output by --version is GPL3' {
	run $em --version
	sanitised_output=$(echo $output)
	[[ "$sanitised_output" =~ "This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>." ]]
}

@test 'fatal warnings are fatal' {
	local warning_text="Hello, world!"
	run --output=stderr -- $em --fatal-warnings << EOF
.warn: $warning_text
EOF
	assert_exit_fail
	[[ "${lines[0]}" =~ [Ee]rror ]]
	[[ ! "${lines[0]}" =~ [Ww]arning ]]
	[[ "${lines[0]}" =~ "$warning_text" ]]
}

@test 'extensions are loaded' {
	local message_text="Hello, world!"
	local ext_file="$BATS_TEST_TMPDIR/ext.lua"
	echo "print('$message_text')" > $ext_file
	run $em -x $ext_file < /dev/null
	assert_exit_pass
	[[ ! "$output" =~ print ]]
	[[ "$output" =~ "$message_text" ]]
}
