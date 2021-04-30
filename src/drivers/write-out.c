#include "write-out.h"

#include "logs/logs.h"
#include <errno.h>
#include <string.h>

int write_output(Str* fname, List* content)
{
	FILE* fp = fopen(fname->str, "w+");
	if (!fp)
	{
		log_err("Could not open file '%s' for writing: %s", fname->str, strerror(errno));
		return 1;
	}

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
			/* log_debug("%p", (void*)(out_buf + curr_pos)); */
			/* log_debug("%s", curr_str->str); */
			/* log_debug("%ld", curr_str->len); */
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

	if (fclose(fp))
	{
		log_err("Failed to close '%s' after writing: %s", fname->str, strerror(errno));
		return 1;
	}
	return 0;
}
