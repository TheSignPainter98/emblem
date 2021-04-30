#include "ast.h"

#include "logs/logs.h"
#include "lua.h"
#include "pp/lambda.h"
#include "style/css.h"
#include <string.h>

void make_doc(Doc* doc, DocTreeNode* root, Args* args)
{
	doc->root	= root;
	doc->styler = malloc(sizeof(Styler));
	make_styler(doc->styler, args);
}

void dest_doc(Doc* doc)
{
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

void make_doc_tree_node_line(DocTreeNode* node, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = LINE;
	content->line = malloc(sizeof(List));

	node->flags	  = 0;
	node->name	  = malloc(sizeof(Str));
	node->style	  = NULL;
	node->content = content;
	node->parent  = NULL;
	node->src_loc = src_loc;

	make_list(content->line);
	make_strc(node->name, NODE_NAME_LINE);
}

void make_doc_tree_node_lines(DocTreeNode* node, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = LINES;
	content->line = malloc(sizeof(List));

	node->flags	  = 0;
	node->name	  = malloc(sizeof(Str));
	node->style	  = NULL;
	node->content = content;
	node->parent  = NULL;
	node->src_loc = src_loc;

	make_list(content->lines);
	make_strc(node->name, NODE_NAME_LINES);
}

void make_doc_tree_node_call(DocTreeNode* node, Str* name, CallIO* call_params, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type		 = CALL;
	content->call_params = call_params;

	node->flags	  = 0;
	node->name	  = name;
	node->style	  = NULL;
	node->content = content;
	node->parent  = NULL;
	node->src_loc = src_loc;

	if (call_params)
	{
		ListIter iter;
		make_list_iter(&iter, call_params->args);
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
			dest_call_io(content->call_params, processing_result);
			free(content->call_params);
			break;
		case LINE:
			dest_list(content->line, true, ed);
			free(content->line);
			break;
		case LINES:
			dest_list(content->lines, true, ed);
			free(content->lines);
			break;
		case PAR:
			dest_list(content->par, true, ed);
			free(content->par);
			break;
		case PARS:
			dest_list(content->pars, true, ed);
			free(content->pars);
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

void make_call_io(CallIO* call_params)
{
	call_params->result = NULL;
	call_params->args	= malloc(sizeof(List));
	make_list(call_params->args);
}

void prepend_call_io_arg(CallIO* call_params, DocTreeNode* arg)
{
	arg->flags |= IS_CALL_PARAM;
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, arg);
	prepend_list_node(call_params->args, ln);
}

void dest_call_io(CallIO* call_params, bool processing_result)
{
	NON_ISO(Destructor ed = ilambda(void, (void* v), { dest_free_doc_tree_node((DocTreeNode*)v, processing_result); }));
	if (call_params->result)
		dest_free_doc_tree_node(call_params->result, true);
	dest_list(call_params->args, true, ed);
	free(call_params->args);
}

Location* dup_loc(Location* todup)
{
	Location* ret	  = malloc(sizeof(Location));
	memcpy(ret, todup, sizeof(Location));
	return ret;
}
