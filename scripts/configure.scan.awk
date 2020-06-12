#!/usr/bin/awk -f

BEGIN {
	pkg_name = "hello-tools"
	pkg_version = 0.3
	pkg_bug_report_address = "https://github.com/TheSignPainter98/emblem-autotools/issues"
	config_headers = "src/config.h"

	func_checks["criterion"] = "criterion_run_all_tests"
	func_checks["spkg"] = "min"
}

/^AC_INIT/ {
	printf "AC_INIT([%s], [%s], [%s])\n", pkg_name, pkg_version, pkg_bug_report_address
	next
}

/^AC_CONFIG_HEADERS/ {
	printf "AC_CONFIG_HEADERS([%s])\n", config_headers
	print "AM_INIT_AUTOMAKE([1.16 foreign subdir-objects dist-xz -Wgnu -Werror])"
	print "AC_PROG_YACC([bison])"
	print "AC_PROG_LEX([flex])"
	print "AC_CONFIG_MACRO_DIRS([m4])"
	print "AM_CONDITIONAL([ANALYZER], [false])"
	print "AC_CONFIG_FILES([Makefile])"
	print "LT_INIT"
	next
}

$2 == "FIXME:" {
	next
}

/^AC_CHECK_LIB/ {
	match($0, /(\[[^\]]*\])/) # Match the package name
	dep_name_raw = substr($0, RSTART, RLENGTH)
	match(dep_name_raw, /[^()]*/)
	dep_name_raw = substr(dep_name_raw, RSTART, RLENGTH)
	dep_name = substr(dep_name_raw, 2, length(dep_name_raw) - 2)

	if (!(dep_name in func_checks)) {
		printf "# FIXME: No check for library %s\n", dep_name
		err = 1
	}
	printf "AC_CHECK_LIB([%s], [%s])\n", dep_name, func_checks[dep_name]
	next
}

1

END {
	exit err
}