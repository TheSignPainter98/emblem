/**
 * @file ast.c
 * @brief Implements the document structure data types
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "ast.h"

#include "logs/logs.h"
#include "lua.h"
#include "pp/lambda.h"
#include <stdlib.h>
#include <string.h>

const char* const node_tree_content_type_names[] = {
	[WORD]	  = "word",
	[CALL]	  = "call",
	[CONTENT] = "content",
};
const size_t node_tree_content_type_names_len
	= sizeof(node_tree_content_type_names) / sizeof(*node_tree_content_type_names);

void make_doc(Doc* doc, DocTreeNode* root, Styler* styler, ExtensionEnv* ext)
{
	doc->root	= root;
	doc->styler = styler;
	doc->ext	= ext;
}

void dest_doc(Doc* doc) { dest_free_doc_tree_node(doc->root, false, CORE_POINTER_DEREFERENCE); }

void make_doc_tree_node_word(DocTreeNode* node, Str* word, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = WORD;
	content->word = word;

	node->flags		= 0;
	node->last_eval = -1;
	node->name		= malloc(sizeof(Str));
	node->style		= NULL;
	node->content	= content;
	node->parent	= NULL;
	node->src_loc	= src_loc;
	node->lp		= NULL;

	make_strc(node->name, NODE_NAME_WORD);
}

void make_doc_tree_node_content(DocTreeNode* node, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type	 = CONTENT;
	content->content = malloc(sizeof(List));

	node->flags		= 0;
	node->last_eval = -1;
	node->name		= malloc(sizeof(Str));
	node->style		= NULL;
	node->content	= content;
	node->parent	= NULL;
	node->src_loc	= src_loc;
	node->lp		= NULL;

	make_list(content->content);
	make_strc(node->name, NODE_NAME_CONTENT);
}

void make_doc_tree_node_call(DocTreeNode* node, Str* name, CallIO* call, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = CALL;
	content->call = call;

	node->flags		= 0;
	node->last_eval = -1;
	node->name		= name;
	node->style		= NULL;
	node->content	= content;
	node->parent	= NULL;
	node->src_loc	= src_loc;
	node->lp		= NULL;

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

void dest_free_doc_tree_node(DocTreeNode* node, bool processing_result, DocTreeNodeSharedDestructionMode shared_mode)
{
	if (shared_mode == LUA_POINTER_DEREFERENCE)
		node->lp = NULL;
	else // shared_mode == CORE_POINTER_DEREFERENCE
		node->parent = NULL;

	// Only free orphaned nodes which aren't referenced from ext-space
	if (node->lp || node->parent)
		return;

	// Call parameters are freed from the call node itself
	if (processing_result && node->flags & IS_CALL_PARAM)
		return;

	if (node->style)
		dest_style(node->style);
	dest_doc_tree_node_content(node->content, processing_result, CORE_POINTER_DEREFERENCE);
	free(node->content);
	free(node->src_loc);
	dest_str(node->name);
	free(node->name);
	free(node);
}

LuaPointer* get_ast_lua_pointer(ExtensionState* s, DocTreeNode* node)
{
	if (!node->lp)
		node->lp = new_lua_pointer(s, AST_NODE, node, true);
	return node->lp;
}

void dest_doc_tree_node_content(
	DocTreeNodeContent* content, bool processing_result, DocTreeNodeSharedDestructionMode shared_mode)
{
	NON_ISO(Destructor ed
		= ilambda(void, (void* v), { dest_free_doc_tree_node((DocTreeNode*)v, processing_result, shared_mode); }));

	switch (content->type)
	{
		case WORD:
			dest_str(content->word);
			free(content->word);
			break;
		case CALL:
			dest_call_io(content->call, processing_result, shared_mode);
			free(content->call);
			break;
		case CONTENT:
			dest_list(content->content, ed);
			free(content->content);
			break;
		default:
			log_err("Failed to destroy content of type %d", content->type);
	}
}

void prepend_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child)
{
	prepend_list(child_list, new_child);
	new_child->parent = parent;
}

void append_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child)
{
	append_list(child_list, new_child);
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
	prepend_list(call->args, arg);
}

void append_call_io_arg(CallIO* call, DocTreeNode* arg)
{
	arg->flags |= IS_CALL_PARAM;
	append_list(call->args, arg);
}

void dest_call_io(CallIO* call, bool processing_result, DocTreeNodeSharedDestructionMode shared_mode)
{
	NON_ISO(Destructor ed
		= ilambda(void, (void* v), { dest_free_doc_tree_node((DocTreeNode*)v, processing_result, shared_mode); }));
	if (call->result)
		dest_free_doc_tree_node(call->result, true, shared_mode);
	dest_list(call->args, ed);
	free(call->args);
}
