#include "linear-formatter.h"

#include "driver-util.h"
#include "write-out.h"
#include <stdarg.h>

void make_linear_formatter(LinearFormatter* formatter, DriverParams* params, size_t num_special_functions,
	const Pair special_functions[num_special_functions], Str* document_output_name_fmt)
{
	// Compute the name of the outputted document
	size_t output_doc_name_len = params->output_stem->len + document_output_name_fmt->len;
	char* output_doc_name_raw  = malloc(output_doc_name_len);
	snprintf(output_doc_name_raw, output_doc_name_len + 1, document_output_name_fmt->str, params->output_stem->str);
	formatter->output_doc_name = malloc(sizeof(Str));
	make_strr(formatter->output_doc_name, output_doc_name_raw);

	// Compute the name of the stylesheet file
	size_t stylesheet_name_len = params->output_stem->len + sizeof(STYLESHEET_NAME_FMT);
	char* stylesheet_name	   = malloc(stylesheet_name_len);
	snprintf(stylesheet_name, stylesheet_name_len + 1, STYLESHEET_NAME_FMT, params->output_stem->str);
	formatter->stylesheet_name = malloc(sizeof(Str));
	make_strr(formatter->stylesheet_name, stylesheet_name);

	formatter->content = malloc(sizeof(List));
	make_list(formatter->content);
	formatter->formatter_content = malloc(sizeof(List));
	make_list(formatter->formatter_content);

	formatter->call_name_map = malloc(sizeof(Map));
	make_map(formatter->call_name_map, hash_str, cmp_strs, (Destructor)dest_free_str);
	for (size_t i = 0; i < num_special_functions; i++)
	{
		Maybe m;
		Str* k = malloc(sizeof(Str));
		make_strv(k, special_functions[i].p0);
		Str* v = malloc(sizeof(Str));
		make_strv(v, special_functions[i].p1);
		push_map(&m, formatter->call_name_map, k, v);
		dest_maybe(&m, NULL);
	}
}

void dest_linear_formatter(LinearFormatter* formatter)
{
	dest_map(formatter->call_name_map, (Destructor)dest_free_str);
	free(formatter->call_name_map);
	dest_list(formatter->formatter_content, true, (Destructor)dest_free_str);
	free(formatter->formatter_content);
	dest_list(formatter->content, true, NULL);
	free(formatter->content);
	dest_str(formatter->output_doc_name);
	free(formatter->output_doc_name);
	dest_str(formatter->stylesheet_name);
	free(formatter->stylesheet_name);
}

void append_linear_formatter_raw(LinearFormatter* formatter, char* raw)
{
	Str* str = malloc(sizeof(Str));
	make_strv(str, raw);
	append_linear_formatter_str(formatter, str);
}

void append_linear_formatter_str(LinearFormatter* formatter, Str* str)
{
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, str);
	append_list_node(formatter->content, ln);
	ListNode* ln2 = malloc(sizeof(ListNode));
	make_list_node(ln2, str);
	append_list_node(formatter->formatter_content, ln2);
}

void append_linear_formatter_strf(LinearFormatter* formatter, char* restrict format, ...)
{
	va_list va;
	va_list va2;
	va_start(va, format);
	va_copy(va2, va);
	size_t maxlen = 1 + vsnprintf(NULL, 0, format, va); // NOLINT
	Str* str	  = malloc(sizeof(Str));
	char* raw	  = malloc(maxlen * sizeof(char));
	vsnprintf(raw, maxlen, format, va2);
	make_strr(str, raw);
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, str);
	append_list_node(formatter->content, ln);
	ListNode* ln2 = malloc(sizeof(ListNode));
	make_list_node(ln2, str);
	append_list_node(formatter->formatter_content, ln2);
	va_end(va2);
	va_end(va);
}

int write_linear_formatter_output(LinearFormatter* formatter)
{
	return write_output(formatter->output_doc_name, formatter->content);
}
