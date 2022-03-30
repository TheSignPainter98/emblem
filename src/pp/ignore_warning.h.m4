#pragma once
m4_divert(`-1')

m4_define(`em4_append', `m4_define(`$1', m4_ifdef(`$1', `m4_defn(`$1')`$3'')`$2')')

m4_define(`x_ignore_warning', `m4_dnl
/**
 * @brief Ignore $3 warnings
 *
 * @param cmd A single line of code where the warning will be ignored
 *
 * @return The line cmd wrapped in warning-ignoring pragmas
 */
#	define $2(cmd) _Pragma("$1 diagnostic push"); _Pragma("$1 diagnostic ignored \"-W$3\""); cmd; _Pragma("$1 diagnostic pop")
')

m4_define(`gcc_ignore_warning', x_ignore_warning(`GCC', $1, $2))
m4_define(`clang_ignore_warning', x_ignore_warning(`clang', $1, $2))

m4_define(`generic_ignore_warning', `m4_dnl
/**
 * @brief Ignore $2 warnings
 *
 * @param cmd A single line of code where the warning will be ignored
 *
 * @return If supported, The line cmd wrapped in warning-ignoring pragmas
 */
#	define $1(cmd) cmd;
')

m4_define(`ignore_warning', `
	em4_append(`gcc_warning_ignorers', `gcc_ignore_warning($1, $2)')
	m4_ifelse($3, `NONE', `
		em4_append(`clang_warning_ignorers', `generic_ignore_warning($1, m4_ifelse($3, `', $2, $3))')
	', `
		em4_append(`clang_warning_ignorers', `clang_ignore_warning($1, m4_ifelse($3, `', $2, $3))')
	')
	em4_append(`generic_warning_ignorers', `generic_ignore_warning($1, $2)')
')

ignore_warning(ARRAY_BOUND_MISMATCH, array-bounds)
ignore_warning(INT_CONVERSION, int-conversion)
ignore_warning(INT_TO_POINTER_CAST, int-to-pointer-cast, int-to-void-pointer-cast)
ignore_warning(MALLOC_LEAK, analyzer-malloc-leak, NONE)
ignore_warning(NON_ISO, pedantic, pedantic)
ignore_warning(POINTER_ARITH, pointer-arith, NONE)
ignore_warning(POINTER_TO_INT_CAST, pointer-to-int-cast, void-pointer-to-int-cast)
ignore_warning(TYPE_PUN_DEREFERENCE, strict-aliasing)

m4_divert(`0')m4_dnl

#if __clang__
#	pragma clang dependency "ignore_warning.h.m4"
clang_warning_ignorers
#elif __GNUC__
#	pragma GCC dependency "ignore_warning.h.m4"
gcc_warning_ignorers
#else
generic_warning_ignorers
#endif
