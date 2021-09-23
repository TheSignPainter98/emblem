/**
 * @file dest-free.h
 * @brief Defines preprocessor rules for the creation of functions which destroy and then free memory of a given type
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#define dest_free_sig(name, type) void dest_free_##name(type* name)
#define dest_free_def(name, type)                                                                                      \
	dest_free_sig(name, type)                                                                                          \
	{                                                                                                                  \
		dest_##name(name);                                                                                             \
		free(name);                                                                                                    \
	}
