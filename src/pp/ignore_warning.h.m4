#pragma once

#if __GNUC__
m4_define(`define_ignore_warning', `m4_dnl
#	define $1(cmd) \
	_Pragma("GCC diagnostic push"); \
	_Pragma("GCC diagnostic ignored \"-W$2\""); \
	cmd; \
	_Pragma("GCC diagnostic pop")m4_dnl
')m4_dnl
define_ignore_warning(MALLOC_LEAK, analyzer-malloc-leak)
define_ignore_warning(NON_ISO, pedantic)
#else
#	define MALLOC_LEAK(cmd) cmd
#	define NON_ISO(cmd)	 cmd
#endif
