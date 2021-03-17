#include "src/data/cmp.h"

#include "src/data/str.h"
#include "src/pp/ignore_warning.h"
#include <criterion/criterion.h>

#define CMP_TEST(name, type, fmt, v1, v2)                                                                              \
	Test(cmp, name)                                                                                                    \
	{                                                                                                                  \
		type rv1 = v1;                                                                                                 \
		type rv2 = v2;                                                                                                 \
		TYPE_PUN_DEREFERENCE(void* a = *(void**)&rv1);                                                                 \
		TYPE_PUN_DEREFERENCE(void* b = *(void**)&rv2);                                                                 \
		cr_assert(cmp_##name##s(a, b) == LT, #name " '" fmt "' >= '" fmt "'", a, b);                                   \
		cr_assert(cmp_##name##s(b, b) == EQ, #name " '" fmt "' != '" fmt "'", b, b);                                   \
		cr_assert(cmp_##name##s(b, a) == GT, #name " '" fmt "' <= '" fmt "'", b, a);                                   \
	}

CMP_TEST(char, char, "%c", 'a', 'b')
CMP_TEST(double, double, "%f", 1234.4321, 5432.5423)
CMP_TEST(float, float, "%f", 0.2f, 10.4f)
CMP_TEST(int, int, "%d", 10, 20)

Test(cmp, ptr)
{
	void* a = (void*)0x1234;
	void* b = (void*)0xabcd;
	cr_assert(cmp_ptrs(a, b) == LT, "ptr %p >= %p", a, b);
	cr_assert(cmp_ptrs(b, b) == EQ, "ptr %p != %p", b, b);
	cr_assert(cmp_ptrs(b, a) == GT, "ptr %p <= %p", b, a);
}

Test(cmp, str)
{
	Str a;
	Str b;
	make_strv(&a, "aaaaa");
	make_strv(&b, "bbbbb");
	cr_assert(cmp_strs(&a, &b) == LT, "str '%s' >= '%s'", a, b);
	cr_assert(cmp_strs(&b, &b) == EQ, "str '%s' != '%s'", a, b);
	cr_assert(cmp_strs(&b, &a) == GT, "str '%s' <= '%s'", a, b);
	dest_str(&a);
	dest_str(&b);
}
