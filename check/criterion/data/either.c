#include "data/either.h"

#include "pp/lambda.h"
#include <criterion/criterion.h>

Test(either, left_memory_life_cycle)
{
	Either e;
	long int val = 104;
	make_either_left(&e, (void*)val);
	cr_assert(e.type == LEFT, "Type of either is not left");
	cr_assert((long int)e.left == val, "Value of left is not what it should be");
	dest_either(&e, NULL, NULL);
}

Test(either, right_memory_life_cycle)
{
	Either e;
	long int val = 104;
	make_either_right(&e, (void*)val);
	cr_assert(e.type == RIGHT, "Type of either is not right");
	cr_assert((long int)e.right == val, "Value of right is not what it should be");
	dest_either(&e, NULL, NULL);
}

Test(either, left_is_not_successful)
{
	Either e;
	long int val = 104;
	make_either_left(&e, (void*)val);
	cr_assert(!succ_either(&e), "Left is successful");
	dest_either(&e, NULL, NULL);
}

Test(either, right_is_successful)
{
	Either e;
	long int val = 104;
	make_either_right(&e, (void*)val);
	cr_assert(succ_either(&e), "Right is unsuccessful");
	dest_either(&e, NULL, NULL);
}

Test(either, left_fmap_unaffected)
{
	Either ei;
	Either eo;
	long int val = 104;
	make_either_left(&ei, (void*)val);
	NON_ISO(
		func_sig(void, f, (void**, void*)) = ilambda(void, (void** o, void* i), { *(long int*)o = (long int)i + 1; }));
	fmap_either(&eo, &ei, f);
	cr_assert((long int)eo.type == ei.type, "Constructor of fmapped either was not the same");
	cr_assert((long int)eo.left == val, "Function was incorrectly applied to either right");
	dest_either(&ei, NULL, NULL);
	dest_either(&eo, NULL, NULL);
}

Test(either, right_fmap_affected)
{
	Either ei;
	Either eo;
	long int val = 104;
	make_either_right(&ei, (void*)val);
	NON_ISO(
		func_sig(void, f, (void**, void*)) = ilambda(void, (void** o, void* i), { *(long int*)o = (long int)i + 1; }));
	long int oval;
	f((void**)&oval, (void*)val);
	fmap_either(&eo, &ei, f);
	cr_assert((long int)eo.type == ei.type, "Constructor of fmapped either was not the same");
	cr_assert((long int)eo.right == oval, "Function was incorrectly applied to either right");
	dest_either(&ei, NULL, NULL);
	dest_either(&eo, NULL, NULL);
}
