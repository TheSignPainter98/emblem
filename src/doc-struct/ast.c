#include "ast.h"

void make_doc(Doc* doc, DocTreeNode* root)
{
	doc->root = root;
	doc->ext_state = NULL;
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
		while(iter_list((void**)&call_param, &iter))
			call_param->parent = node;

		dest_list_iter(&iter);
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
	call_params->args = malloc(sizeof(List));
	make_list(call_params->args);
}

void prepend_call_io_arg(CallIO* call_params, DocTreeNode* arg)
{
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, arg);
	prepend_list_node(call_params->args, ln);
}
