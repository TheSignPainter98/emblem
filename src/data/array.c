#include "array.h"
#include "../pp/ignore_warning.h"

bool make_arr(Array* arr, size_t cnt)
{
	if (!arr)
		return false;
	MALLOC_LEAK(arr->data = (void**)calloc(cnt, sizeof(void*)))
	arr->cnt = cnt;
	return !!arr->data;
}

void dest_arr(Array* arr)
{
	if (!arr)
		return;
	free(arr->data);
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
