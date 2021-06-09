#include "discern-pars.h"

#include "logs/logs.h"

typedef enum
{
	NO_PAR_NODE,
	MAYBE_CHILD_PAR_NODE,
	REQUIRES_PAR_NODE,
} ParNodeRequirement;

static void discern_pars_beneath_node(DocTreeNode* node);
static void apply_par_node(ListNode* containingNode, DocTreeNode* node);
static ParNodeRequirement requires_par_node(DocTreeNode* node);

void discern_pars(Doc* doc)
{
	log_info("Marking paragraphs");
	if (doc->root)
		discern_pars_beneath_node(doc->root);
}

static void discern_pars_beneath_node(DocTreeNode* node)
{
	switch (node->content->type)
	{
		case WORD:
			return;
		case CALL:
			if (node->content->call->result)
				discern_pars_beneath_node(node->content->call->result);
			return;
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
					case MAYBE_CHILD_PAR_NODE:
						discern_pars_beneath_node(childNode);
						break;
					case REQUIRES_PAR_NODE:
						apply_par_node(listNode, childNode);
						break;
				}
			}

			dest_list_iter(&li);
			return;
		}
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
	prepend_call_io_arg(call_io, node);
	make_doc_tree_node_call(pnode, pcall, call_io, loc);
	call_io->result = node;

	containingNode->data = pnode;
	pnode->parent		 = node->parent;
	node->parent		 = pnode;
}

static ParNodeRequirement requires_par_node(DocTreeNode* node)
{
	if (node->flags & DISQUALIFIED_PARAGRAPH)
		return NO_PAR_NODE;

	switch (node->content->type)
	{
		case CALL:
			return MAYBE_CHILD_PAR_NODE;
		case CONTENT:
			if (!(node->flags & PARAGRAPH_CANDIDATE))
				return NO_PAR_NODE;
			switch (node->content->content->cnt)
			{
				case 0:
					return NO_PAR_NODE;
				case 1:
				{
					DocTreeNode* sole_child = node->content->content->fst->data;
					if (sole_child->content->type == CALL)
						return MAYBE_CHILD_PAR_NODE;
					return REQUIRES_PAR_NODE;
				}
				default:
					return REQUIRES_PAR_NODE;
			}
		default:
			return NO_PAR_NODE;
	}
}
