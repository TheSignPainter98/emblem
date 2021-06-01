#pragma once

#define dest_free_sig(name, type) void dest_free_##name(type* name)
#define dest_free_def(name, type)                                                                                                \
	dest_free_sig(name, type)                                                                                                \
	{                                                                                                                  \
		dest_##name(name);                                                                                               \
		free(name);                                                                                                      \
	}
