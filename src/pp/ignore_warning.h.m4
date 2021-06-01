#pragma once

#if __GNUC__ && !__clang__
#	pragma GCC dependency "ignore_warning.h.m4"
m4_define(`define_ignore_warning', `m4_dnl
/**
 * @brief Ignore $2 warnings
 *
 * @param cmd A single line of code where the warning will be ignored
 *
 * @return If supported, The line cmd wrapped in warning-ignoring pragmas
 */
#	define $1(cmd) \
	_Pragma("GCC diagnostic push"); \
	_Pragma("GCC diagnostic ignored \"-W$2\""); \
	cmd; \
	_Pragma("GCC diagnostic pop")m4_dnl
')m4_dnl
define_ignore_warning(ARRAY_BOUND_MISMATCH, array-bounds)
define_ignore_warning(INT_CONVERSION, int-conversion)
define_ignore_warning(INT_TO_POINTER_CAST, int-to-pointer-cast)
define_ignore_warning(MALLOC_LEAK, analyzer-malloc-leak)
define_ignore_warning(NON_ISO, pedantic)
define_ignore_warning(POINTER_TO_INT_CAST, pointer-to-int-cast)
define_ignore_warning(TYPE_PUN_DEREFERENCE, strict-aliasing)
#else
m4_define(`define_dumb_ignore_warning', `m4_dnl
/**
 * @brief Ignore $2 warnings
 *
 * @param cmd A single line of code where the warning will be ignored
 *
 * @return If supported, The line cmd wrapped in warning-ignoring pragmas
 */
#	define $1(cmd) cmd;
')m4_dnl
define_dumb_ignore_warning(ARRAY_BOUND_MISMATCH, array-bounds)
define_dumb_ignore_warning(INT_CONVERSION, int-conversion)
define_dumb_ignore_warning(INT_TO_POINTER_CAST, int-to-pointer-cast)
define_dumb_ignore_warning(MALLOC_LEAK, analyzer-malloc-leak)
define_dumb_ignore_warning(NON_ISO, pedantic)
define_dumb_ignore_warning(POINTER_TO_INT_CAST, pointer-to-int-cast)
define_dumb_ignore_warning(TYPE_PUN_DEREFERENCE, strict-aliasing)
#endif
