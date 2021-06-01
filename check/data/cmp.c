#include "src/data/cmp.h"

#include "src/data/str.h"
#include "src/pp/ignore_warning.h"
#include <criterion/criterion.h>

#define CMP_INTEGRAL_TEST(name, type, fmt, v1, v2)                                                                     \
	Test(cmp, name)                                                                                                    \
	{                                                                                                                  \
		INT_CONVERSION(void* a = (type)v1);                                                                            \
		INT_CONVERSION(void* b = (type)v2);                                                                            \
		cr_assert(cmp_##name##s(a, b) == CMP_LT, #name " '" fmt "' >= '" fmt "'", a, b);                               \
		cr_assert(cmp_##name##s(b, b) == CMP_EQ, #name " '" fmt "' != '" fmt "'", b, b);                               \
		cr_assert(cmp_##name##s(b, a) == CMP_GT, #name " '" fmt "' <= '" fmt "'", b, a);                               \
	}

#define CMP_FLOAT_TEST(name, type, fmt, v1, v2)                                                                        \
	Test(cmp, name)                                                                                                    \
	{                                                                                                                  \
		type rv1 = v1;                                                                                                 \
		type rv2 = v2;                                                                                                 \
		void* a[1];                                                                                                    \
		void* b[1];                                                                                                    \
		*a = (void*)&rv1;                                                                                              \
		*b = (void*)&rv1;                                                                                              \
		cr_assert(cmp_##name##s(*a, *b) == CMP_LT, #name " '" fmt "' >= '" fmt "'", v1, v2);                           \
		cr_assert(cmp_##name##s(*b, *b) == CMP_EQ, #name " '" fmt "' != '" fmt "'", v2, v1);                           \
		cr_assert(cmp_##name##s(*b, *a) == CMP_GT, #name " '" fmt "' <= '" fmt "'", v2, v1);                           \
	}

// BEGIN_NOLINT
CMP_INTEGRAL_TEST(char, char, "%c", 'a', 'b')
CMP_INTEGRAL_TEST(int, int, "%d", 10, 20)
CMP_FLOAT_TEST(double, double, "%f", 1234.4321, 5432.5423)
CMP_FLOAT_TEST(float, float, "%f", 0.2f, 10.4f)
// END_NOLINT

Test(cmp, ptr)
{
	void* a = (void*)0x1234;
	void* b = (void*)0xabcd;
	cr_assert(cmp_ptrs(a, b) == CMP_LT, "ptr %p >= %p", a, b);
	cr_assert(cmp_ptrs(b, b) == CMP_EQ, "ptr %p != %p", b, b);
	cr_assert(cmp_ptrs(b, a) == CMP_GT, "ptr %p <= %p", b, a);
}

Test(cmp, str)
{
	Str a;
	Str b;
	make_strv(&a, "aaaaa");
	make_strv(&b, "bbbbb");
	cr_assert(cmp_strs(&a, &b) == CMP_LT, "str '%s' >= '%s'", a, b);
	cr_assert(cmp_strs(&b, &b) == CMP_EQ, "str '%s' != '%s'", a, b);
	cr_assert(cmp_strs(&b, &a) == CMP_GT, "str '%s' <= '%s'", a, b);
	dest_str(&a);
	dest_str(&b);
}

Test(cmp, streq)
{
	char* ss[] = {
		"Hello, world!",
		"Hello, world!",
		"How are you?",
	};
	cr_assert(streq(ss[0], ss[0]), "Pointers to the same string are not reported as equal");
	cr_assert(streq(ss[0], ss[1]), "Pointers to equal strings are not recognised as such");
	cr_assert_not(streq(ss[0], ss[2]), "Pointers to non-equal strings are not recognised as such");
}
