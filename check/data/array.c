#include "src/data/array.h"

#include <criterion/criterion.h>

Test(array, memory_life_cycle)
{
	Array arr;
	size_t cnt = 10;
	make_arr(&arr, cnt);
	cr_assert(arr.cnt == cnt, "Array length was not as expected");
	cr_assert(arr.cnt == cnt, "Array length was not as expected");
	dest_arr(&arr);
}

Test(array, initial_memory_is_empty)
{
	Array arr;
	size_t cnt = 100;
	make_arr(&arr, cnt);
	for (size_t i = 0; i < cnt; i++)
	{
		Maybe m;
		get_arrv(&m, &arr, i);
		cr_assert(succ_maybe(&m), "Array get was not successful");
		cr_assert(m.just == 0, "Array index %d was not zero", i);
	}
}

Test(array, get_set_normal_function)
{
	Array arr;
	size_t cnt = 3;
	make_arr(&arr, cnt);
	void* val = (void*)100;
	bool r = set_arrv(&arr, 1, val);
	cr_assert(r, "Valid array setting did not return true");
	Maybe m;
	get_arrv(&m, &arr, 1);
	cr_assert(m.just == val);
}

Test(array, cannot_get_or_set_bad_indices)
{
	Array arr;
	size_t cnt = 10;
	make_arr(&arr, cnt);
	bool r = set_arrv(&arr, 100, (void*)100);
	cr_assert_not(r, "Array set returned true when index was bad");
	Maybe m;
	get_arrv(&m, &arr, 100);
	cr_assert_not(succ_maybe(&m), "Got successful result type when indexing array with bad index");
}
