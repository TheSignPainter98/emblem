#!/bin/bash

_em=(./em -Ttxt --colourise-output=-1 -o-)
em=${_em[@]}

function assert_exit_pass() {
	[[ $status -eq 0 ]]
}

function assert_exit_fail() {
	[[ $status -ne 0 ]]
}
