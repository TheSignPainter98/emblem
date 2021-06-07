#include "ast.h"

#include "logs/logs.h"
#include "lua.h"
#include "pp/lambda.h"
#include "style/css.h"
#include <stdlib.h>
#include <string.h>

const char* const node_tree_content_type_names[] = {
	[WORD]	  = "word",
	[CALL]	  = "call",
	[CONTENT] = "content",
};
const size_t node_tree_content_type_names_len
	= sizeof(node_tree_content_type_names) / sizeof(*node_tree_content_type_names);

void make_doc(Doc* doc, DocTreeNode* root, Args* args)
{
	doc->root	= root;
	doc->styler = malloc(sizeof(Styler));
	make_styler(doc->styler, args);

	ExtParams ext_params;
	init_ext_params(&ext_params, args, doc->styler);
	doc->ext = malloc(sizeof(ExtensionEnv));
	make_ext_env(doc->ext, &ext_params);
}

void dest_doc(Doc* doc)
{
	dest_ext_env(doc->ext);
	free(doc->ext);
	dest_styler(doc->styler);
	free(doc->styler);
	dest_free_doc_tree_node(doc->root, false);
}

void make_doc_tree_node_word(DocTreeNode* node, Str* word, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = WORD;
	content->word = word;

	node->flags	  = 0;
	node->name	  = malloc(sizeof(Str));
	node->style	  = NULL;
	node->content = content;
	node->parent  = NULL;
	node->src_loc = src_loc;

	make_strc(node->name, NODE_NAME_WORD);
}

void make_doc_tree_node_content(DocTreeNode* node, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type	 = CONTENT;
	content->content = malloc(sizeof(List));

	node->flags	  = 0;
	node->name	  = malloc(sizeof(Str));
	node->style	  = NULL;
	node->content = content;
	node->parent  = NULL;
	node->src_loc = src_loc;

	make_list(content->content);
	make_strc(node->name, NODE_NAME_CONTENT);
}

void make_doc_tree_node_call(DocTreeNode* node, Str* name, CallIO* call, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = CALL;
	content->call = call;

	node->flags	  = 0;
	node->name	  = name;
	node->style	  = NULL;
	node->content = content;
	node->parent  = NULL;
	node->src_loc = src_loc;

	if (call)
	{
		ListIter iter;
		make_list_iter(&iter, call->args);
		DocTreeNode* call_param;
		while (iter_list((void**)&call_param, &iter))
			call_param->parent = node;

		dest_list_iter(&iter);
	}
}

void dest_free_doc_tree_node(DocTreeNode* node, bool processing_result)
{
	if (processing_result && node->flags & IS_CALL_PARAM)
		return;

	if (node->style)
		dest_style(node->style);
	dest_doc_tree_node_content(node->content, processing_result);
	free(node->content);
	free(node->src_loc);
	dest_str(node->name);
	free(node->name);
	free(node);
}

void dest_doc_tree_node_content(DocTreeNodeContent* content, bool processing_result)
{
	NON_ISO(Destructor ed = ilambda(void, (void* v), { dest_free_doc_tree_node((DocTreeNode*)v, processing_result); }));

	switch (content->type)
	{
		case WORD:
			dest_str(content->word);
			free(content->word);
			break;
		case CALL:
			dest_call_io(content->call, processing_result);
			free(content->call);
			break;
		case CONTENT:
			dest_list(content->content, true, ed);
			free(content->content);
			break;
		default:
			log_err("Failed to destroy content of type %d", content->type);
	}
}

void prepend_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child)
{
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, new_child);
	prepend_list_node(child_list, ln);

	new_child->parent = parent;
}

void make_call_io(CallIO* call)
{
	call->result = NULL;
	call->args	 = malloc(sizeof(List));
	make_list(call->args);
}

void prepend_call_io_arg(CallIO* call, DocTreeNode* arg)
{
	arg->flags |= IS_CALL_PARAM;
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, arg);
	prepend_list_node(call->args, ln);
}

void dest_call_io(CallIO* call, bool processing_result)
{
	NON_ISO(Destructor ed = ilambda(void, (void* v), { dest_free_doc_tree_node((DocTreeNode*)v, processing_result); }));
	if (call->result)
		dest_free_doc_tree_node(call->result, true);
	dest_list(call->args, true, ed);
	free(call->args);
}
