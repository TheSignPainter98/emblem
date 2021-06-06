#include "write-out.h"

#include "logs/logs.h"
#include <errno.h>
#include <stdio.h>
#include <string.h>

int write_output(Str* fmt, Str* stem, bool allow_stdout, List* content)
{
	if (allow_stdout && streq(stem->str, "-"))
		return write_output_to_file(stdout, content);

	size_t output_doc_name_len = 1 + snprintf(NULL, 0, fmt->str, stem->str);
	char output_doc_name_raw[output_doc_name_len];
	snprintf(output_doc_name_raw, output_doc_name_len + 1, fmt->str, stem->str);
	Str output_doc_name;
	make_strv(&output_doc_name, output_doc_name_raw);

	int rc = write_output_to_path(&output_doc_name, content);

	dest_str(&output_doc_name);

	return rc;
}

int write_output_to_path(Str* fname, List* content)
{
	FILE* fp = fopen(fname->str, "w+");
	if (!fp)
	{
		log_err("Could not open file '%s' for writing: %s", fname->str, strerror(errno));
		return 1;
	}

	log_info("Writing output to '%s'", fname->str);
	write_output_to_file(fp, content);

	if (fclose(fp))
	{
		log_err("Failed to close '%s' after writing: %s", fname->str, strerror(errno));
		return 1;
	}
	return 0;
}

int write_output_to_file(FILE* fp, List* content)
{
	size_t out_buf_size = 0;
	ListIter sli;
	make_list_iter(&sli, content);
	Str* curr;
	while (iter_list((void**)&curr, &sli))
		out_buf_size += curr->len;
	dest_list_iter(&sli);

	char* out_buf = calloc(1 + out_buf_size, sizeof(char));
	if (out_buf)
	{
		// Collect all into single buffer
		size_t curr_pos = 0;
		ListIter cli;
		make_list_iter(&cli, content);
		Str* curr_str;
		while (iter_list((void**)&curr_str, &cli))
		{
			strncat(out_buf + curr_pos, curr_str->str, curr_str->len);
			curr_pos += curr_str->len;
		}
		out_buf[out_buf_size] = '\0';

		// Output all at once
		fputs(out_buf, fp);
		free(out_buf);
	}
	else
	{
		ListIter cli;
		make_list_iter(&cli, content);
		Str* curr_str;
		while (iter_list((void**)&curr_str, &cli))
			fputs(curr_str->str, fp);
		dest_list_iter(&cli);
	}

	return 0;
}
