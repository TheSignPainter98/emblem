#!/usr/bin/awk -f

BEGIN {
	open = 0
	printf "{\"name\":\"%s\",\"lines\":[[1,", file
}

/\/\/\s*BEGIN_NOLINT/ {
	open = 1
	printf "%d],[", NR
}

/\/\/\s*END_NOLINT/ {
	open = 0
	printf "%d,", NR
}

END {
	printf "%d]]},", NR
	if (open) {
		printf "Still open in file %s\n", file >"/dev/stderr"
		exit 1
	}
}
