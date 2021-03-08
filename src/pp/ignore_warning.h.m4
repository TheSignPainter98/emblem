#pragma once

#if __GNUC__
m4_define(`define_ignore_warning', `m4_dnl
#	define $1(cmd) \
	_Pragma("GCC diagnostic push"); \
	_Pragma("GCC diagnostic ignored \"-W$2\""); \
	cmd; \
	_Pragma("GCC diagnostic pop")m4_dnl
')m4_dnl
define_ignore_warning(INT_TO_POINTER_CAST, int-to-pointer-cast)
define_ignore_warning(MALLOC_LEAK, analyzer-malloc-leak)
define_ignore_warning(NON_ISO, pedantic)
define_ignore_warning(POINTER_TO_INT_CAST, pointer-to-int-cast)
define_ignore_warning(TYPE_PUN_DEREFERENCE, strict-aliasing)
#else
#	define INT_TO_POINTER_CAST(cmd) cmd
#	define MALLOC_LEAK(cmd) cmd
#	define NON_ISO(cmd) cmd
#	define POINTER_TO_INT_CAST(cmd) cmd
#	define TYPE_PUN_DEREFERENCE(cmd) cmd
#endif
