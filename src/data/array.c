#include "array.h"

#include "pp/ignore_warning.h"

bool make_arr(Array* arr, size_t cnt)
{
	if (!arr)
		return false;
	MALLOC_LEAK(arr->data = (void**)calloc(cnt, sizeof(void*)))
	arr->cnt = cnt;
	return !!arr->data;
}

void dest_arr(Array* arr, func_sig(void, ed, (void*)))
{
	if (!arr)
		return;

	if (ed)
		for (size_t i = 0; i < arr->cnt; i++)
			ed(arr->data[i]);

	free(arr->data);
	arr->cnt = 0;
}

void get_arrv(Maybe* ret, Array* arr, size_t idx)
{
	if (arr->cnt <= idx)
		make_maybe_nothing(ret);
	else
		make_maybe_just(ret, arr->data[idx]);
}

bool set_arrv(Array* arr, size_t idx, void* val)
{
	if (arr->cnt <= idx)
		return false;

	arr->data[idx] = val;
	return true;
}

void make_arr_from_list(Array* arr, List* l)
{
	make_arr(arr, l->cnt);
	ListNode* curr = l->fst;
	for (size_t i = 0; i < l->cnt; i++)
	{
		arr->data[i] = curr->data;
		curr = curr->nxt;
	}
}
