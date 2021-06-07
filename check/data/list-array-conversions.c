#include "src/data/list-array-conversions.h"

#include <criterion/criterion.h>

Test(list, make_from_array)
{
	const size_t arrlen = 100;
	List l;
	Array arr;
	make_arr(&arr, arrlen);

	for (size_t i = 0; i < arrlen; i++)
		set_arrv(&arr, i, (void*)i);

	make_list_from_arr(&l, &arr);

	cr_assert(
		l.cnt == arr.cnt, "List created from array had different length, expected %ld but got %ld", l.cnt, arr.cnt);

	ListIter iter;
	make_list_iter(&iter, &l);
	void* val;
	size_t i = 0;
	while (iter_list(&val, &iter))
	{
		cr_assert((size_t)val == i, "Incorrect value in list from array, expected %ld but got %ld", i, (size_t)val);
		i++;
	}

	dest_list(&l, NULL);
	dest_arr(&arr, NULL);
}

Test(array, make_from_list)
{
	const size_t llen = 100;
	Array arr;
	List list;
	make_list(&list);

	for (size_t i = 0; i < llen; i++)
		append_list(&list, (void*)i);

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
	dest_list(&list, NULL);
}
