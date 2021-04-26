#include "src/data/str.h"

#include "src/pp/ignore_warning.h"
#include <criterion/criterion.h>

Test(str, memory_cycle)
{
	Str s;
	make_str(&s);
	cr_assert(s.str == NULL, "Initial string content is not null");
	cr_assert(s.len == 0, "Initial empty string does not have zero length");
	cr_assert_not(s.free_mem, "String memory marked to be free when initialised as empty");
	dest_str(&s);
}

Test(str, make_from_raw)
{
	Str s;
	char str_content[] = "Hello, world!";
	make_strv(&s, str_content);
	cr_assert(!strcmp(s.str, str_content), "Stored string has different length than what was input, expected '%s' but got '%s'", str_content, s.str);
	cr_assert(s.len == strlen(str_content), "Stored string reported different length field than was input, expected %d but got %d", strlen(str_content), s.len);
	cr_assert_not(s.free_mem, "Memory marked to be freed from non-copied string");
	dest_str(&s);
}

Test(str, make_copied_raw)
{
	Str s;
	char str_content[] = "Hello, world!";
	make_strc(&s, str_content);
	cr_assert(!strcmp(s.str, str_content), "Stored string has different length than what was input, expected '%s' but got '%s'", str_content, s.str);
	cr_assert(s.len == strlen(str_content), "Stored string reported different length field than was input, expected %d but got %d", strlen(str_content), s.len);
	cr_assert(s.free_mem, "Memory marked to be freed from non-copied string");
	dest_str(&s);
}

Test(str, make_copied_raw_reference)
{
	Str s;
	char str_content[] = "Hello, world!";
	make_strr(&s, str_content);
	cr_assert(!strcmp(s.str, str_content), "Stored string has different length than what was input, expected '%s' but got '%s'", str_content, s.str);
	cr_assert(s.len == strlen(str_content), "Stored string reported different length field than was input, expected %d but got %d", strlen(str_content), s.len);
	cr_assert(s.free_mem, "Memory marked to be freed from non-copied string");
	dest_str(&s);
}

Test(str, make_length)
{
	Str s;
	const size_t str_len = 100;
	bool rc = make_strl(&s, str_len);
	cr_assert(rc, "Making string of fixed length reported failure when none was expected");
	cr_assert(s.len == str_len);
	for (size_t i = 0; i <= str_len; i++)
	{
		cr_assert_not(s.str[i], "String initial memory at index %ld was not zero", i);
	}
	cr_assert(s.free_mem, "Memory not marked to be freed in empty-generated string");
	dest_str(&s);
}

Test(str, to_arr)
{
	Str s;
	make_strl(&s, 100);
	const size_t str_len = s.len;
	Array arr;
	str_to_arr(&arr, &s);
	cr_assert(arr.cnt == str_len, "Array generated from string had wrong length, expected %ld but got %ld", str_len, arr.cnt);
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
	const size_t arr_len = sizeof(arr_content) / sizeof(*arr_content);
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
		POINTER_TO_INT_CAST(cr_assert(s.str[i] == (char)arr.data[i], "String and array values differed at index %ld, expected %c but got '%c'", i, (char)arr.data[i], s.str[i]));
	}
	cr_assert(!s.str[arr_len], "Generated string was not null-terminated");
	dest_arr(&arr, NULL);
	dest_str(&s);
}

Test(str, get_char)
{
	Str s;
	char str_content[] = "Hello, world!";
	const size_t str_len = strlen(str_content);
	make_strv(&s, str_content);
	for (size_t i = 0; i < str_len; i++)
	{
		Maybe m;
		get_strc(&m, &s, i);
		cr_assert(m.type == JUST, "Valid index character-get returned nothing");
		POINTER_TO_INT_CAST(cr_assert((char)m.just == s.str[i], "String-char get returned the wrong value, expected %c but got '%c'", s.str[i], (char)m.just));
		dest_maybe(&m, NULL);
	}
	Maybe m;
	get_strc(&m, &s, str_len + 10);
	cr_assert(m.type == NOTHING, "Getting string char at incorrect index seemed to return something");
	dest_maybe(&m, NULL);
	dest_str(&s);
}

Test(str, set_char)
{
	Str s;
	char str_content[] = "Hello, world!";
	const size_t str_len = strlen(str_content);
	make_strv(&s, str_content);
	for (size_t i = 0; i < str_len; i++)
	{
		bool rc = set_strc(&s, i, (char)i);
		INT_TO_POINTER_CAST(cr_assert(s.str[i] == (char)i, "String-char setting did not change value as required, expected '%p' but got '%p'", (void*)s.str[i], (void*)i));
		cr_assert(rc, "String-char setting returned false when successful operation occurred");
	}
	cr_assert_not(set_strc(&s, str_len + 10, 'a'), "Setting char value at incorrect index did not indicate error");
	dest_str(&s);
}

Test(str, copy_into_enough_space)
{
	Str s1;
	Str s2;
	char s1Content[] = "Hello mighty fine world how are you?";
	char s2Content[] = "YYYYYYYYY";
	make_strv(&s1, s1Content);
	make_strv(&s2, s2Content);

	const size_t copyStartIdx = 12;
	bool rc = copy_into_str(&s1, &s2, copyStartIdx);
	cr_assert(rc, "Copy into string apparently failed with valid parameters");
	for (size_t i = 0; i < s1.len; i++)
	{
		if (copyStartIdx <= i && i < copyStartIdx + s2.len)
		{
			cr_assert(s1.str[i] == s2Content[i - copyStartIdx], "Container string did not have content from inserted string at index %d <= %d < %d, expected '%c' but got '%c'", copyStartIdx, i, copyStartIdx + s2.len, s2Content[i - copyStartIdx], s1.str[i]);
		}
		else
		{
			cr_assert(s1.str[i] == s1Content[i], "Container string did not have content from inserted string at index %d, expected '%c' but got '%c'", i, s1Content[i], s1.str[i]);
		}
	}

	dest_str(&s1);
	dest_str(&s2);
}

Test(str, copy_into_too_little_space)
{
	Str s1;
	Str s2;
	char s1Content[] = "Tiny string";
	char s2Content[] = "Some massive string which is way to large to be copied into the other";
	make_strv(&s1, s1Content);
	make_strv(&s2, s2Content);
	cr_assert_not(copy_into_str(&s1, &s2, 0), "Copying into a too-small string was successful");
	dest_str(&s1);
	dest_str(&s2);
}
