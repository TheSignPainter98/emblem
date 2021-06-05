#include "cmp.h"

#include "data/str.h"
#include "pp/ignore_warning.h"
#include <string.h>

#define COMPARISON_BODY(type)                                                                                          \
	{                                                                                                                  \
		TYPE_PUN_DEREFERENCE(type p1 = *(type*)&v1);                                                                   \
		TYPE_PUN_DEREFERENCE(type p2 = *(type*)&v2);                                                                   \
		return p1 < p2 ? CMP_LT : p1 == p2 ? CMP_EQ : CMP_GT;                                                          \
	}
#if __clang__
#	define CMP(name, type) Cmp(fun cmp_##name##s)(void*, void*) = fun Cmp(void* v1, void* v2) COMPARISON_BODY(type);
#elif __GNUC__
/**
 * @brief Create a generic comparator function
 *
 * @param name Name of the type
 * @param type Type to compare
 *
 * @return A function which compares two `type`s
 */
#	define CMP(name, type) Cmp cmp_##name##s(void* v1, void* v2) COMPARISON_BODY(type)
#endif

CMP(char, char)
CMP(double, double)
CMP(float, float)
CMP(int, int)
CMP(size_t, size_t)
CMP(ptr, void*)

#if !__clang__
Cmp cmp_strs(void* v1, void* v2)
{
	Str* s1 = v1;
	Str* s2 = v2;
	return strcmp(s1->str, s2->str);
}
#endif

bool streq(char const* s, char const* t) { return !strcmp(s, t); }
