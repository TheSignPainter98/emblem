/**
 * @file list.c
 * @brief Implements the list data-structure, a deque
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "list.h"

#include "pp/unused.h"
#include <stddef.h>

static void make_internal_list_node(ListNode* ln, void* data);

void make_list(List* l)
{
	l->fst	   = NULL;
	l->lst	   = NULL;
	l->cnt	   = 0;
	l->own_mem = true;
}

void dest_list(List* l, Destructor ed)
{
	if (l->own_mem)
	{
		ListNode* curr = l->fst;
		while (curr)
		{
			ListNode* nxt = curr->nxt;
			dest_list_node(curr, ed);

			if (curr->list_mem)
				free(curr);

			curr = nxt;
		}
	}
}

void set_sublist(List* l, bool is_sublist) { l->own_mem = !is_sublist; }

void make_list_node(ListNode* ln, void* data)
{
	ln->nxt		 = NULL;
	ln->prv		 = NULL;
	ln->data	 = data;
	ln->list_mem = false;
}

static void make_internal_list_node(ListNode* ln, void* data)
{
	ln->nxt		 = NULL;
	ln->prv		 = NULL;
	ln->data	 = data;
	ln->list_mem = true;
}

void dest_list_node(ListNode* ln, Destructor ed)
{
	if (ed)
		ed(ln->data);
}

bool append_list(List* l, void* v)
{
	ListNode* ln = malloc(sizeof(ListNode));
	make_internal_list_node(ln, v);
	return append_list_node(l, ln); // NOLINT
}

bool append_list_node(List* l, ListNode* ln)
{
	if (!l || !ln)
		return false;

	ln->prv = l->lst;
	ln->nxt = NULL;

	if (!l->fst)
		l->fst = ln;

	if (l->lst)
		l->lst->nxt = ln;
	l->lst = ln;

	l->cnt++;

	return true;
}

bool prepend_list(List* l, void* v)
{
	ListNode* ln = malloc(sizeof(ListNode));
	make_internal_list_node(ln, v);
	return prepend_list_node(l, ln); // NOLINT
}

bool prepend_list_node(List* l, ListNode* ln)
{
	if (!l || !ln)
		return false;

	ln->nxt = l->fst;
	ln->prv = NULL;

	if (l->fst)
		l->fst->prv = ln;
	l->fst = ln;

	if (!l->lst)
		l->lst = ln;

	l->cnt++;

	return true;
}

bool remove_list_node(List* l, ListNode* ln)
{
	if (!l || !ln)
		return false;

	if (ln->nxt)
		ln->nxt->prv = ln->prv;
	if (ln->prv)
		ln->prv->nxt = ln->nxt;

	if (l->fst == ln)
		l->fst = ln->nxt;

	if (l->lst == ln)
		l->lst = ln->prv;

	ln->nxt = NULL;
	ln->prv = NULL;

	l->cnt--;

	return true;
}

void make_list_iter(ListIter* i, List* l) { i->nxt = l->fst; }

void make_reversed_list_iter(ReversedListIter* i, List* l) { i->nxt = l->lst; }

void dest_list_iter(ListIter* i) { UNUSED(i); }

void dest_reversed_list_iter(ReversedListIter* i) { UNUSED(i); }

bool iter_list(void** val, ListIter* i) {
	ListNode* ln;
	bool succ = iter_list_nodes(&ln, i);
	if (succ)
		*val = ln->data;
	return succ;
}

bool iter_list_nodes(ListNode** n, ListIter* i)
{
	bool succ = i->nxt;
	if (succ)
	{
		*n	   = i->nxt;
		i->nxt = i->nxt->nxt;
	}
	else
		*n = NULL;

	return !!succ;
}

bool iter_list_reversed(void** val, ReversedListIter* i)
{
	bool succ = i->nxt;
	if (succ)
	{
		*val   = i->nxt->data;
		i->nxt = i->nxt->prv;
	}
	else
		*val = NULL;

	return !!succ;
}

bool in_list(List* l, void* val)
{
	Maybe m;
	in_list_eq(&m, l, cmp_size_ts, val);
	bool rv = m.type == JUST;
	dest_maybe(&m, NULL);
	return rv;
}

void in_list_eq(Maybe* m, List* l, Comparator cmp, void* val)
{
	ListNode* curr = l->fst;

	while (curr)
	{
		if (!cmp(val, curr->data))
		{
			make_maybe_just(m, curr);
			return;
		}
		curr = curr->nxt;
	}

	make_maybe_nothing(m);
}

void concat_list(List* r, List* l1, List* l2)
{
	r->own_mem = true;
	r->cnt	   = l1->cnt + l2->cnt;
	r->fst	   = NULL;
	if (!r->cnt || (!l1->fst && !l2->fst))
	{
		r->lst = NULL;
		return;
	}

	List* l			   = l1->fst ? l1 : l2;
	ListNode* curr	   = l->fst;
	ListNode* prv	   = NULL;
	ListNode* new_curr = NULL;
	while (curr)
	{
		new_curr = malloc(sizeof(ListNode));
		if (!r->fst)
			r->fst = new_curr;
		make_internal_list_node(new_curr, curr->data);
		new_curr->prv = prv;
		if (prv)
			prv->nxt = new_curr;

		prv	 = new_curr;
		curr = curr->nxt;
		if (!curr && l != l2)
		{
			l	 = l2;
			curr = l2->fst;
		}
	}
	new_curr->nxt = NULL;
	r->lst		  = new_curr;
}

void cconcat_list(List* r, List* l)
{
	r->cnt += l->cnt;

	ListNode* curr	 = l->fst;
	ListNode* app	 = r->lst;
	ListNode* fst_ln = NULL;
	while (curr)
	{
		ListNode* ln = malloc(sizeof(ListNode));
		make_internal_list_node(ln, curr->data);
		if (app)
			app->nxt = ln;
		if (!fst_ln)
			fst_ln = ln;
		ln->prv = app;
		app		= ln;
		curr	= curr->nxt;
	}
	r->lst = app;
	if (!r->fst)
		r->fst = fst_ln;
}

void iconcat_list(List* r, List* l)
{
	l->own_mem = false;
	r->cnt += l->cnt;
	if (r->lst)
		r->lst->nxt = l->fst;
	if (l->fst)
		l->fst->prv = r->lst;
	r->lst = l->lst;
}

bool all_list(List* l)
{
	if (!l)
		return true;

	bool r		   = true;
	ListNode* curr = l->fst;

	while (curr)
	{
		r &= !!curr->data;
		curr = curr->nxt;
	}

	return r;
}

bool any_list(List* l)
{
	if (!l)
		return false;

	bool r		   = false;
	ListNode* curr = l->fst;

	while (curr)
	{
		r |= !!curr->data;
		curr = curr->nxt;
	}

	return r;
}

bool is_empty_list(List* l) { return l->cnt == 0; }
