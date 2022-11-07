/**
 * @file str.c
 * @brief Implements the string data-structure
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "str.h"

#include <string.h>

void make_strv(Str* str, const char* raw)
{
	*(const char**)&str->str = raw;
	*(size_t*)&str->len		 = strlen(raw);
	*(bool*)&str->free_mem	 = false;
	str->lwc_rep			 = NULL;
}

void make_strr(Str* str, const char* raw)
{
	*(const char**)&str->str = raw;
	*(size_t*)&str->len		 = strlen(raw);
	*(bool*)&str->free_mem	 = true;
	str->lwc_rep			 = NULL;
}

void make_strc(Str* str, const char* raw)
{
	*(char**)&str->str	   = strdup(raw);
	*(size_t*)&str->len	   = strlen(raw);
	*(bool*)&str->free_mem = true;
	str->lwc_rep		   = NULL;
}

void dest_str(Str* str)
{
	if (str->free_mem)
		free((void*)str->str);
	if (str->lwc_rep)
		lwc_string_unref(str->lwc_rep);
}

void str_to_arr(Array* arr, Str* str)
{
	make_arr(arr, str->len);
	for (size_t i = 0; i < str->len; i++)
	{
		INT_TO_POINTER_CAST(arr->data[i] = (void*)str->str[i]); // NOLINT
	}
}

void arr_to_str(Str* str, Array* arr)
{
	char* s = calloc(arr->cnt + 1, sizeof(char));
	for (size_t i = 0; i < arr->cnt; i++)
		s[i] = *(char*)&arr->data[i];
	s[arr->cnt] = '\0';
	make_strr(str, s);
}

void get_strc(Maybe* ret, Str* str, size_t idx)
{
	if (str->len <= idx)
		make_maybe_nothing(ret);
	else
		make_maybe_just(ret, *(void**)&str->str[idx]);
}

void dup_str(Str* o, Str* todup)
{
	*(bool*)&o->free_mem = true;
	*(size_t*)&o->len	 = todup->len;
	*(char**)&o->str	 = malloc((todup->len + 1) * sizeof(char));
	o->lwc_rep			 = NULL;
	memcpy((void*)o->str, todup->str, todup->len + 1);
}

lwc_string* get_lwc_string(Str* s)
{
	if (!s->lwc_rep)
		lwc_intern_string(s->str, s->len, &s->lwc_rep);

	return s->lwc_rep;
}

dest_free_def(str, Str)
