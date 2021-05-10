#include "src/data/array.h"

#include <criterion/criterion.h>
#include <stdio.h>
#include <stdlib.h>

Test(array, memory_life_cycle)
{
	Array arr;
	size_t cnt = 10;
	make_arr(&arr, cnt);
	cr_assert(arr.cnt == cnt, "Array length was not as expected");
	cr_assert(arr.cnt == cnt, "Array length was not as expected");
	dest_arr(&arr, NULL);
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
	dest_arr(&arr, NULL);
}

Test(array, get_set_normal_function)
{
	Array arr;
	size_t cnt = 3;
	make_arr(&arr, cnt);
	void* val = (void*)100;
	bool r	  = set_arrv(&arr, 1, val);
	cr_assert(r, "Valid array setting did not return true");
	Maybe m;
	get_arrv(&m, &arr, 1);
	cr_assert(m.just == val);
	dest_arr(&arr, NULL);
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
	dest_arr(&arr, NULL);
}

Test(array, conversion_from_list)
{
	const size_t llen = 100;
	Array arr;
	List list;
	make_list(&list);

	for (size_t i = 0; i < llen; i++)
	{
		ListNode* ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)i);
		append_list_node(&list, ln);
	}

	make_arr_from_list(&arr, &list);

	cr_assert(arr.cnt == list.cnt, "Array created from list had a different length, got %ld but expected %ld", arr.cnt,
		list.cnt);

	for (size_t i = 0; i < llen; i++)
	{
		Maybe m;
		get_arrv(&m, &arr, i);
		cr_assert(m.type == JUST, "Array read returned object with nothing constructor");
		cr_assert((size_t)m.just == i, "Incorrect value in array, expected %ld but got %ld", i, (size_t)m.just);
	}

	dest_arr(&arr, NULL);
	dest_list(&list, true, NULL);
}

Test(array, iter_memory_life_cycle)
{
	Array a;
	make_arr(&a, 10);
	ArrayIter iter;
	make_arr_iter(&iter, &a);
	dest_arr_iter(&iter);
	dest_arr(&a, NULL);
}

Test(array, iter)
{
	Array a;
	const size_t arrLen = 100;
	make_arr(&a, arrLen);
	for (size_t i = 0; i < arrLen; i++)
		a.data[i] = (void*)i;

	ArrayIter iter;
	size_t itered = 0;
	void* elem;
	make_arr_iter(&iter, &a);
	while (iter_arr(&elem, &iter))
	{
		cr_assert((size_t)elem == itered, "Array oterator returned incorrect value, expected %ld but got %ld", itered,
			(size_t)elem);
		itered++;
	}
	cr_assert(itered == arrLen, "Iterated too few elements, expected %ld but only oterated over %ld", arrLen, itered);

	dest_arr_iter(&iter);
	dest_arr(&a, NULL);
}
