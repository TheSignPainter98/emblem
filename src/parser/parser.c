/**
 * @file parser.c
 * @brief Implements the parser at the top-level (entire-document)
 * @author Edward Jones
 * @date 2021-09-17
 */
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

static void foster_body_node(DocTreeNode** root);

void parse_doc(Maybe* mo, Locked* mtNamesList, Args* args, const char* input)
{
	log_info("Parsing document '%s'", args->input_file);

	// Select whether to use the core parser, or to use an extension one
	bool use_core_parser = true;
	char* dialect		 = args->input_driver;
	if (*args->input_driver)
		use_core_parser = streq(args->input_driver, "em");
	else
	{
		char* ext = strrchr(input, '.');
		if (ext++ && strcmp(ext, "em"))
		{
			use_core_parser = false;
			dialect			= ext;
		}
	}

	if (use_core_parser)
	{
		unsigned int nerrs = parse_file(mo, mtNamesList, args, input);

		if (mo->type == NOTHING)
		{
			log_err("Parsing document '%s' failed with %d error%s.", input, nerrs, nerrs - 1 ? "s" : "");
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
		make_strv(fname_str, input);
		make_strv(tname_str, dialect);
		make_strv(call_name, "include");

		make_location(loc, 0, 0, 1, 0, src_name, false);

		make_doc_tree_node_word(fname_node, fname_str, loc);
		make_doc_tree_node_word(tname_node, tname_str, dup_loc(loc, false));

		make_call_io(call);
		append_call_io_arg(call, fname_node);
		append_call_io_arg(call, tname_node);

		make_doc_tree_node_call(root, call_name, call, dup_loc(loc, false));

		make_maybe_just(mo, root);
	}

	if (mo->type == NOTHING)
		return;
	DocTreeNode* root = mo->just;

	foster_body_node(&root);

	root->parent = root;
}

static void foster_body_node(DocTreeNode** root)
{
	DocTreeNode* old_root = *root;
	*root				  = malloc(sizeof(DocTreeNode));
	CallIO* call		  = malloc(sizeof(CallIO));
	Str* body_str		  = malloc(sizeof(Str));
	make_strv(body_str, ROOT_NODE_NAME);
	make_call_io(call);
	append_call_io_arg(call, old_root);
	make_doc_tree_node_call(*root, body_str, call, dup_loc(old_root->src_loc, false));
}
