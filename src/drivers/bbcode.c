#include "bbcode.h"

#include "data/tuple.h"
#include "doc-struct/ast.h"
#include "linear-formatter.h"
#include "logs/logs.h"

#define BBCODE_DOCUMENT_OUTPUT_NAME_FMT "%s.bb"

static Pair const bbcode_special_functions[] = {
	{ "bf", "b" },
	{ "it", "i" },
	{ "sc", "u" },
	{ "af", "s" },
	{ "pre", "pre" },
	{ "quote", "quote" },
	{ "tt", "tt" },
	{ "ul", "list type=decimal" },
	{ "ol", "list" },
	{ "li", "li" },
	{ "img", "img" },
	{ "sub", "sub" },
	{ "sup", "sup" },
	{ "url", "url" },
	{ "more", "more" },
	{ "spoiler", "spoiler" },
	{ "hr", "hr" },
	{ "justify", "justify" },
	{ "left", "left" },
	{ "centre", "center" },
	{ "center", "center" },
	{ "right", "right" },
};
static size_t const num_bbcode_special_functions = sizeof(bbcode_special_functions) / sizeof(*bbcode_special_functions);

static int driver_runner(Doc* doc, DriverParams* params);
static int format_doc_as_bbcode(LinearFormatter* formatter, Doc* doc);
static int format_node_as_bbcode(LinearFormatter* formatter, DocTreeNode* node);
static int format_node_list_as_bbcode(LinearFormatter* formatter, List* node_list);

int make_bbcode_driver(InternalDriver* driver)
{
	OutputDriverInf* driver_inf = malloc(sizeof(OutputDriverInf));
	driver_inf->support			= TS_BASIC_STYLING;

	driver->name = "bbcode";
	driver->inf	 = driver_inf;
	driver->run	 = driver_runner;

	return 0;
}

static int driver_runner(Doc* doc, DriverParams* params)
{
	int rc;
	LinearFormatter formatter;
	Str document_output_name_fmt;
	make_strv(&document_output_name_fmt, BBCODE_DOCUMENT_OUTPUT_NAME_FMT);
	make_linear_formatter(
		&formatter, params, num_bbcode_special_functions, bbcode_special_functions, &document_output_name_fmt);

	rc = format_doc_as_bbcode(&formatter, doc);
	if (rc)
		return rc;

	rc = write_linear_formatter_output(&formatter, true);

	dest_linear_formatter(&formatter);
	return rc;
}

static int format_doc_as_bbcode(LinearFormatter* formatter, Doc* doc)
{
	return format_node_as_bbcode(formatter, doc->root);
}

static int format_node_as_bbcode(LinearFormatter* formatter, DocTreeNode* node)
{
	switch (node->content->type)
	{
		case WORD:
			ListNode* ln = malloc(sizeof(ListNode));
			make_list_node(ln, node->content->word);
			append_list_node(formatter->content, ln);
			append_linear_formatter_raw(formatter, " ");
			return 0;
		case CALL:
		{
			int rc = 0;
			Maybe m;
			get_map(&m, formatter->call_name_map, node->name);
			if (node->content->call_params->result)
			{
				if (m.type == JUST)
				{
					Str* bbcode_node_header_content = m.just;
					append_linear_formatter_strf(formatter, "[%s]", bbcode_node_header_content->str);
					rc = format_node_as_bbcode(formatter, node->content->call_params->result);
					append_linear_formatter_strf(formatter, "[/%s]", ((Str*)m.just)->str);
				}
				else
					rc = format_node_as_bbcode(formatter, node->content->call_params->result);
			}
			dest_maybe(&m, NULL);
			return rc;
		}
		case LINE:
		{
			int rc = format_node_list_as_bbcode(formatter, node->content->line);
			return rc;
		}
		case LINES:
			return format_node_list_as_bbcode(formatter, node->content->lines);
		case PAR:
		{
			int rc = format_node_list_as_bbcode(formatter, node->content->par);
			append_linear_formatter_raw(formatter, "\n\n");
			return rc;
		}
		case PARS:
			return format_node_list_as_bbcode(formatter, node->content->pars);
		default:
			log_err("Unknown node content type: %d", node->content->type);
			return 1;
	}
}

static int format_node_list_as_bbcode(LinearFormatter* formatter, List* node_list)
{
	int rc = 0;

	ListIter li;
	make_list_iter(&li, node_list);
	DocTreeNode* node;
	while (iter_list((void**)&node, &li))
	{
		rc = format_node_as_bbcode(formatter, node);
		if (rc)
			break;
	}
	dest_list_iter(&li);
	return rc;
}
