/**
 * @file discern-pars.c
 * @brief Implements functions to automatically place paragraph nodes in a document
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "discern-pars.h"

#include "logs/logs.h"
#include "style/selection-engine.h"
#include <string.h>

typedef enum
{
	NO_PAR_NODE,
	MAYBE_CHILD_REQUIRES_PAR_NODE,
	REQUIRES_PAR_NODE,
} ParFosterRequirement;

static void apply_par_node(ListNode* containingNode, DocTreeNode* node);
static ParFosterRequirement requires_par_node(DocTreeNode* node);
static bool has_non_null_content(DocTreeNode* node);

int introduce_foster_pars(DocTreeNode* node)
{
	switch (node->content->type)
	{
		case WORD:
			return 0;
		case CALL:
			if (node->content->call->result)
				introduce_foster_pars(node->content->call->result);
			return 0;
		case CONTENT:
		{
			ListIter li;
			make_list_iter(&li, node->content->content);
			ListNode* listNode;
			while (iter_list_nodes(&listNode, &li))
			{
				DocTreeNode* childNode = listNode->data;
				switch (requires_par_node(childNode))
				{
					case NO_PAR_NODE:
						break;
					case MAYBE_CHILD_REQUIRES_PAR_NODE:
						introduce_foster_pars(childNode);
						break;
					case REQUIRES_PAR_NODE:
						apply_par_node(listNode, childNode);
						break;
				}
			}

			dest_list_iter(&li);
			return 0;
		}
		default:
			log_err_at(node->src_loc, "Failed to introduce foster paragraphs, encountered node of unknown type %d",
				node->content->type);
			return -1;
	}
}

static void apply_par_node(ListNode* containingNode, DocTreeNode* node)
{
	DocTreeNode* pnode = malloc(sizeof(DocTreeNode));
	Str* pcall		   = malloc(sizeof(Str));
	make_strv(pcall, "p");
	Location* loc	= dup_loc(node->src_loc);
	CallIO* call_io = malloc(sizeof(CallIO));
	make_call_io(call_io);

	// Create par-node and set it as the parent of 'node'
	DocTreeNode* tmp_parent = node->parent;
	prepend_call_io_arg(call_io, node);
	make_doc_tree_node_call(pnode, pcall, call_io, loc);
	containingNode->data = pnode;
	pnode->parent		 = tmp_parent;
	call_io->result		 = node;
	pnode->flags |= IS_GENERATED_NODE;

	// Update the prev_sibling of the next element
	if (containingNode->nxt)
	{
		DocTreeNode* nxt_node = containingNode->nxt->data;
		if (nxt_node)
			nxt_node->prev_sibling = pnode;
	}

	// Update the previous siblings
	pnode->prev_sibling = node->prev_sibling;
	node->prev_sibling	= NULL;

	if (node->style_data->node_css_data)
		modify_node_data(node, NODE_DATA_ANCESTORS_MODIFIED);
}

static ParFosterRequirement requires_par_node(DocTreeNode* node)
{
	if (node->flags & DISQUALIFIED_PARAGRAPH)
		return NO_PAR_NODE;

	switch (node->content->type)
	{
		case CALL:
			return MAYBE_CHILD_REQUIRES_PAR_NODE;
		case CONTENT:
			if (node->flags & INCLUDED_FILE_ROOT && node->content->content->cnt)
				return MAYBE_CHILD_REQUIRES_PAR_NODE;
			switch (node->content->content->cnt)
			{
				case 0:
					return NO_PAR_NODE;
				case 1:
				{
					DocTreeNode* sole_child = node->content->content->fst->data;
					if (sole_child->content->type == CALL || sole_child->flags & INCLUDED_FILE_ROOT)
						return MAYBE_CHILD_REQUIRES_PAR_NODE;
					if (sole_child->content->type == CONTENT && sole_child->content->content->cnt == 1)
					{
						// Case to handle .include directives with single-line directives, alone on a line
						DocTreeNode* sole_child_sole_child = sole_child->content->content->fst->data;
						if (sole_child_sole_child->content->type == CALL
							|| sole_child_sole_child->flags & INCLUDED_FILE_ROOT)
							return MAYBE_CHILD_REQUIRES_PAR_NODE;
					}
					return REQUIRES_PAR_NODE;
				}
				default:
					return has_non_null_content(node) ? REQUIRES_PAR_NODE : NO_PAR_NODE;
			}
		default:
			return NO_PAR_NODE;
	}
}

static bool has_non_null_content(DocTreeNode* node)
{
	if (!node)
		return false;
	switch (node->content->type)
	{
		case WORD:
			return *node->content->word->str;
		case CALL:
			return has_non_null_content(node->content->call->result);
		case CONTENT:
			ListIter li;
			make_list_iter(&li, node->content->content);
			DocTreeNode* child;
			while (iter_list((void**)&child, &li))
				if (has_non_null_content(child))
					return true;
			return false;
		default:
			log_warn_at(node->src_loc, "Unknown content type %d", node->content->type);
			return false;
	}
}
