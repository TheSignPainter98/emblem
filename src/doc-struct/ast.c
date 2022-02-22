/**
 * @file ast.c
 * @brief Implements the document structure data types
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "ast.h"

#include "logs/logs.h"
#include "lua.h"
#include "parser/sanitise-word.h"
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
	doc->tdoc	= NULL;
	doc->styler = styler;
	doc->ext	= ext;
}

void dest_doc(Doc* doc) { dest_free_doc_tree_node(doc->root, false); }

void make_doc_tree_node_word(DocTreeNode* node, Str* raw, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = WORD;
	content->word = malloc(sizeof(Word));
	make_word(content->word, raw, src_loc);

	node->flags		   = 0;
	node->last_eval	   = -1;
	node->name		   = malloc(sizeof(Str));
	node->style_name   = node->name;
	node->style		   = NULL;
	node->style_data   = malloc(sizeof(StyleData));
	node->content	   = content;
	node->parent	   = NULL;
	node->prev_sibling = NULL;
	node->src_loc	   = src_loc;

	make_strc(node->name, NODE_NAME_WORD);
	make_style_data(node->style_data, node->style_name, node);
}

void make_doc_tree_node_content(DocTreeNode* node, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type	 = CONTENT;
	content->content = malloc(sizeof(List));

	node->flags		   = 0;
	node->last_eval	   = -1;
	node->name		   = malloc(sizeof(Str));
	node->style_name   = node->name;
	node->style		   = NULL;
	node->style_data   = malloc(sizeof(StyleData));
	node->content	   = content;
	node->parent	   = NULL;
	node->prev_sibling = NULL;
	node->src_loc	   = src_loc;

	make_list(content->content);
	make_strc(node->name, NODE_NAME_CONTENT);
	make_style_data(node->style_data, node->style_name, node);
}

void make_doc_tree_node_call(DocTreeNode* node, Str* name, CallIO* call, Location* src_loc)
{
	DocTreeNodeContent* content = malloc(sizeof(DocTreeNodeContent));

	content->type = CALL;
	content->call = call;

	node->flags		   = 0;
	node->last_eval	   = -1;
	node->name		   = name;
	node->style_name   = malloc(sizeof(Str));
	node->style		   = NULL;
	node->style_data   = malloc(sizeof(StyleData));
	node->content	   = content;
	node->parent	   = NULL;
	node->prev_sibling = NULL;
	node->src_loc	   = src_loc;

	const char* s = name->str;
	char* t = malloc(1 + name->len);
	{
		size_t i = 0;
		while (*s)
		{
			if (*s == '_')
				t[i++] = '-';
			else if (*s != '*')
				t[i++] = *s;
			s++;
		}
		t[i] = '\0';
	}
	make_strr(node->style_name, t);

	make_style_data(node->style_data, node->style_name, node);

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
	dest_style_data(node->style_data);
	free(node->style_data);
	dest_doc_tree_node_content(node->content, processing_result);
	free(node->content);
	free(node->src_loc);
	if (node->style_name != node->name)
	{
		dest_str(node->style_name);
		free(node->style_name);
	}
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
			dest_word(content->word);
			free(content->word);
			break;
		case CALL:
			dest_call_io(content->call, processing_result);
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
	if (child_list->fst)
		((DocTreeNode*)(child_list->fst->data))->prev_sibling = new_child;
	prepend_list(child_list, new_child);
	new_child->parent = parent;
}

void connect_to_parent(DocTreeNode* child, DocTreeNode* parent)
{
	if (parent && parent->content->type == CONTENT)
		append_doc_tree_node_child(parent, parent->content->content, child);
	else
		child->parent = parent;
}

void append_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child)
{
	if (child_list->lst)
		new_child->prev_sibling = child_list->lst->data;
	append_list(child_list, new_child);
	new_child->parent = parent;
}

void make_word(Word* word, Str* raw, Location* src_loc)
{
	word->raw = raw;
	sanitise_word(word, src_loc);
}

void make_call_io(CallIO* call)
{
	call->result = NULL;
	call->attrs	 = NULL;
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

void make_attrs(Attrs* attrs) { make_map(attrs, hash_str, cmp_strs, (Destructor)dest_free_str); }

void dest_attrs(Attrs* attrs) { dest_map(attrs, (Destructor)dest_free_str); }

void dest_free_attrs(Attrs* attrs) { dest_attrs(attrs); free(attrs); }

int set_attr(Attrs* attrs, Str* k, Str* v)
{
	if (!attrs)
		return 1;
	int rc = 0;
	Maybe old;
	push_map(&old, attrs, k, v);
	if (old.type == JUST)
	{
		dest_free_str((Str*)old.just);
		rc = 1;
	}
	return rc;
}

void get_attr(Maybe* ret, Attrs* attrs, Str* k)
{
	if (attrs)
		get_map(ret, attrs, k);
	else
		make_maybe_nothing(ret);
}

void dest_word(Word* word)
{
	dest_free_str(word->raw);
	dest_free_str(word->sanitised);
}

void dest_call_io(CallIO* call, bool processing_result)
{
	NON_ISO(Destructor ed = ilambda(void, (void* v), { dest_free_doc_tree_node((DocTreeNode*)v, processing_result); }));
	if (call->result)
		dest_free_doc_tree_node(call->result, true);
	dest_list(call->args, ed);
	free(call->args);
}
