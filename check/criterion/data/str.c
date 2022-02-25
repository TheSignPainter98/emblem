#include "src/data/str.h"

#include "src/pp/ignore_warning.h"
#include <criterion/criterion.h>

Test(str, make_from_raw)
{
	Str s;
	char str_content[] = "Hello, world!";
	make_strv(&s, str_content);
	cr_assert(!strcmp(s.str, str_content),
		"Stored string has different length than what was input, expected '%s' but got '%s'", str_content, s.str);
	cr_assert(s.len == strlen(str_content),
		"Stored string reported different length field than was input, expected %d but got %d", strlen(str_content),
		s.len);
	cr_assert_not(s.free_mem, "Memory marked to be freed from non-copied string");
	dest_str(&s);
}

Test(str, make_copied_raw)
{
	Str s;
	char str_content[] = "Hello, world!";
	make_strc(&s, str_content);
	cr_assert(!strcmp(s.str, str_content),
		"Stored string has different length than what was input, expected '%s' but got '%s'", str_content, s.str);
	cr_assert(s.len == strlen(str_content),
		"Stored string reported different length field than was input, expected %d but got %d", strlen(str_content),
		s.len);
	cr_assert(s.free_mem, "Memory marked to be freed from non-copied string");
	dest_str(&s);
}

Test(str, make_copied_raw_reference)
{
	Str s;
	char* str_content = strdup("Hello, world!");
	make_strr(&s, str_content);
	cr_assert(!strcmp(s.str, str_content),
		"Stored string has different length than what was input, expected '%s' but got '%s'", str_content, s.str);
	cr_assert(s.len == strlen(str_content),
		"Stored string reported different length field than was input, expected %d but got %d", strlen(str_content),
		s.len);
	cr_assert(s.free_mem, "Memory marked to be freed from non-copied string");
	dest_str(&s);
}

Test(str, to_arr)
{
	Str s;
	make_strv(&s, "hfdjka fhdsjka fhdjsa fhdsja fjkjh123'{}");
	const size_t str_len = s.len;
	Array arr;
	str_to_arr(&arr, &s);
	cr_assert(
		arr.cnt == str_len, "Array generated from string had wrong length, expected %ld but got %ld", str_len, arr.cnt);
	for (size_t i = 0; i < str_len; i++)
	{
		POINTER_TO_INT_CAST(cr_assert((char)arr.data[i] == s.str[i]));
	}

	dest_arr(&arr, NULL);
	dest_str(&s);
}

Test(str, from_arr)
{
	Str s;
	Array arr;
	const char arr_content[] = { 'H', 'e', 'l', 'l', 'o', ',', ' ', 'W', 'o', 'r', 'l', 'd', '!' };
	const size_t arr_len	 = sizeof(arr_content) / sizeof(*arr_content);
	make_arr(&arr, arr_len);
	for (size_t i = 0; i < arr_len; i++)
	{
		INT_TO_POINTER_CAST(arr.data[i] = (void*)arr_content[i]);
	}
	arr_to_str(&s, &arr);
	cr_assert(s.len == arr_len, "String and array sizes differ, expected %ld but got %ld\n", arr_len, s.len);
	cr_assert(s.free_mem, "String not marked to free memory when generated from array");
	for (size_t i = 0; i < arr_len; i++)
	{
		POINTER_TO_INT_CAST(cr_assert(s.str[i] == (char)arr.data[i],
			"String and array values differed at index %ld, expected %c but got '%c'", i, (char)arr.data[i], s.str[i]));
	}
	cr_assert(!s.str[arr_len], "Generated string was not null-terminated");
	dest_arr(&arr, NULL);
	dest_str(&s);
}

Test(str, get_char)
{
	Str s;
	char str_content[]	 = "Hello, world!";
	const size_t str_len = strlen(str_content);
	make_strv(&s, str_content);
	for (size_t i = 0; i < str_len; i++)
	{
		Maybe m;
		get_strc(&m, &s, i);
		cr_assert(m.type == JUST, "Valid index character-get returned nothing");
		POINTER_TO_INT_CAST(cr_assert((char)m.just == s.str[i],
			"String-char get returned the wrong value, expected %c but got '%c'", s.str[i], (char)m.just));
		dest_maybe(&m, NULL);
	}
	Maybe m;
	get_strc(&m, &s, str_len + 10);
	cr_assert(m.type == NOTHING, "Getting string char at incorrect index seemed to return something");
	dest_maybe(&m, NULL);
	dest_str(&s);
}
