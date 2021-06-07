#include "parser.h"

#include "data/list.h"
#include "data/locked.h"
#include "doc-struct/location.h"
#include "emblem-parser.h"
#include "logs/logs.h"
#include "pp/unused.h"
#include <stdbool.h>
#include <stdio.h>

static void hang_pars(Doc* doc);
static bool is_par_content(DocTreeNode* node);

void parse_doc(Maybe* mo, List* namesList, Args* args)
{
	Locked mtNamesList;
	make_locked(&mtNamesList, namesList);
	log_info("Parsing document '%s'", args->input_file);
	parse_file(mo, &mtNamesList, args, args->input_file);
	dest_locked(&mtNamesList, NULL);

	if (mo->type == JUST)
		hang_pars(mo->just);
}

static void hang_pars(Doc* doc)
{
	if (!doc->root)
		return;

	ListIter li;
	make_list_iter(&li, doc->root->content->content);

	List* new_root_children = malloc(sizeof(List));
	make_list(new_root_children);

	DocTreeNode* node;
	while (iter_list((void**)&node, &li))
	{
		if (!is_par_content(node))
			continue;

		DocTreeNode* pnode = malloc(sizeof(DocTreeNode));
		Str* pcall		   = malloc(sizeof(Str));
		make_strv(pcall, "p");
		Location* loc	= dup_loc(node->src_loc);
		CallIO* call_io = malloc(sizeof(CallIO));
		make_call_io(call_io);
		prepend_call_io_arg(call_io, node);
		make_doc_tree_node_call(pnode, pcall, call_io, loc);

		append_list(new_root_children, pnode);

		pnode->parent = doc->root;
		node->parent  = pnode;
	}

	dest_list_iter(&li);
	dest_list(doc->root->content->content, NULL);
	free(doc->root->content->content);
	doc->root->content->content = new_root_children;
}

static bool is_par_content(DocTreeNode* node)
{
	UNUSED(node);
	// When floating figures are made, this will need to recurse down to the first word or floating block
	return true;
}
