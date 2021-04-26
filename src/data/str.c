#include "str.h"

#include <string.h>

void make_str(Str* str)
{
	*(char**)&str->str = NULL;
	*(size_t*)&str->len = 0;
	*(bool*)&str->free_mem = false;
}

void make_strv(Str* str, char* raw)
{
	*(char**)&str->str = raw;
	*(size_t*)&str->len = strlen(raw);
	*(bool*)&str->free_mem = false;
}

void make_strr(Str* str, char* raw)
{
	*(char**)&str->str = raw;
	*(size_t*)&str->len = strlen(raw);
	*(bool*)&str->free_mem = true;
}

void make_strc(Str* str, char* raw)
{
	*(char**)&str->str = strdup(raw);
	*(size_t*)&str->len = strlen(raw);
	*(bool*)&str->free_mem = true;
}

bool make_strl(Str* str, size_t len)
{
	*(char**)&str->str = calloc(len + 1, sizeof(char));
	*(size_t*)&str->len = len;
	*(bool*)&str->free_mem = true;

	return !!str->str;
}

void dest_str(Str* str)
{
	if (str->free_mem)
		free((void*)str->str);
}

void str_to_arr(Array* arr, Str* str)
{
	make_arr(arr, str->len);
	fprintf(stderr, "%ld\n", str->len);
	for (size_t i = 0; i < str->len; i++)
	{
		INT_TO_POINTER_CAST(arr->data[i] = (void*)str->str[i]);
	}
}

void arr_to_str(Str* str, Array* arr)
{
	make_strl(str, arr->cnt);
	for (size_t i = 0; i < arr->cnt; i++)
		str->str[i] = *(char*)&arr->data[i];
}

void get_strc(Maybe* ret, Str* str, size_t idx)
{
	if (str->len <= idx)
		make_maybe_nothing(ret);
	else
		make_maybe_just(ret, *(void**)&str->str[idx]);
}

bool set_strc(Str* str, size_t idx, char val)
{
	if (str->len <= idx)
		return false;

	str->str[idx] = val;

	return true;
}

bool copy_into_str(Str* cont, Str* ins, size_t startIdx)
{
	if (cont->len <= ins->len + startIdx)
		return false;

	for (size_t i = 0; i < ins->len; i++)
		cont->str[startIdx + i] = ins->str[i];

	return true;
}
