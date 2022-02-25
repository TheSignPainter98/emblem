#include "data/maybe.h"

#include <criterion/criterion.h>

Test(maybe, nothing_memory_life_cycle)
{
	Maybe m;
	make_maybe_nothing(&m);
	cr_assert(m.type == NOTHING, "Type of maybe is not nothing");
	cr_assert(m.nothing == UNIT, "Value of maybe-nothing is not the unit");
	dest_maybe(&m, NULL);
}

Test(maybe, just_memory_life_cycle)
{
	Maybe m;
	long int val = 105;
	make_maybe_just(&m, (void*)val);
	cr_assert(m.type == JUST, "Type of maybe is not just");
	cr_assert((long int)m.just == val, "Value of maybe-just is not %ld, got %ld instead", val, (long int)m.just);
	dest_maybe(&m, NULL);
}

Test(maybe, nothing_is_not_successful)
{
	Maybe m;
	make_maybe_nothing(&m);
	cr_assert(!succ_maybe(&m), "Nothing is successful");
	dest_maybe(&m, NULL);
}

Test(maybe, just_is_successful)
{
	Maybe m;
	make_maybe_just(&m, (void*)100);
	cr_assert(succ_maybe(&m), "Just is unsuccessfil");
	dest_maybe(&m, NULL);
}

Test(maybe, fmap_nothing)
{
	Maybe mi;
	Maybe mo;
	func_sig(void, f, (void**, void*)) = NULL;
	make_maybe_nothing(&mi);
	fmap_maybe(&mo, &mi, f);
	cr_assert((long int)mo.nothing == UNIT, "Fmap was incorrectly applied to the unit");
	dest_maybe(&mi, NULL);
	dest_maybe(&mo, NULL);
}

Test(maybe, fmap_just)
{
	Maybe mi;
	Maybe mo;
	long int val = 104;
	NON_ISO(
		func_sig(void, f, (void**, void*)) = ilambda(void, (void** o, void* i), { *(long int*)o = (long int)i + 1; }));
	make_maybe_just(&mi, (void*)val);
	fmap_maybe(&mo, &mi, f);
	long int oval;
	f((void**)&oval, (void*)val);
	cr_assert((long int)mo.just == oval, "Fmap was incorrectly applied to the stored value");
	dest_maybe(&mi, NULL);
	dest_maybe(&mo, NULL);
}
