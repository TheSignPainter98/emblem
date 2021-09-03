#include "parser.h"

#include "data/cmp.h"
#include "data/list.h"
#include "data/locked.h"
#include "doc-struct/location.h"
#include "emblem-parser.h"
#include "logs/logs.h"
#include "pp/unused.h"
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

void parse_doc(Maybe* mo, Locked* mtNamesList, Args* args)
{
	log_info("Parsing document '%s'", args->input_file);

	// Select whether to use the core parser, or to use an extension one
	bool use_core_parser = true;
	char* dialect = args->input_driver;
	if (*args->input_driver)
		use_core_parser = streq(args->input_driver, "em");
	else
	{
		char* ext = strrchr(args->input_file, '.');
		if (ext++ && strcmp(ext, "em"))
		{
			use_core_parser = false;
			dialect = ext;
		}
	}

	if (use_core_parser)
	{
		unsigned int nerrs = parse_file(mo, mtNamesList, args, args->input_file);

		if (mo->type == NOTHING)
		{
			make_maybe_nothing(mo);
			log_err("Parsing document '%s' failed with %d error%s.", args->input_file, nerrs, nerrs - 1 ? "s" : "");
		}
	}
	else
	{
		// Construct the document .include{fname}{dialect}
		DocTreeNode* root		= malloc(sizeof(DocTreeNode));
		CallIO* call			= malloc(sizeof(CallIO));
		DocTreeNode* fname_node = malloc(sizeof(DocTreeNode));
		DocTreeNode* tname_node = malloc(sizeof(DocTreeNode));
		Str* fname_str			= malloc(sizeof(Str));
		Str* tname_str			= malloc(sizeof(Str));
		Location* loc			= malloc(sizeof(Location));
		Str* src_name			= malloc(sizeof(Str));
		Str* call_name			= malloc(sizeof(Str));
		make_strv(src_name, "cli");
		USE_LOCK(List * names_list, mtNamesList, append_list(names_list, src_name));
		make_strv(fname_str, args->input_file);
		make_strv(tname_str, dialect);
		make_strv(call_name, "include");

		loc->first_line	  = 1;
		loc->first_column = 1;
		loc->last_line	  = 1;
		loc->last_column  = 1;
		loc->src_file	  = src_name;

		make_doc_tree_node_word(fname_node, fname_str, loc);
		make_doc_tree_node_word(tname_node, tname_str, dup_loc(loc));

		make_call_io(call);
		append_call_io_arg(call, fname_node);
		append_call_io_arg(call, tname_node);

		make_doc_tree_node_call(root, call_name, call, dup_loc(loc));

		make_maybe_just(mo, root);
	}
}
