#include "list.h"

#include "../logs/logs.h"

void deque_node_create(DequeNode* n, void* val);
void deque_node_destroy(DequeNode* n);

bool deque_create(Deque* dq)
{
	if (!dq)
	{
		log_err("Attempted to initialise null deque");
		return false;
	}
	dq->fst = NULL;
	dq->lst = NULL;
	return true;
}

bool deque_destroy(Deque* dq)
{
	if (!dq)
	{
		log_err("Attempted to destroy null deque\n");
		return false;
	}

	DequeNode* curr = dq->fst;
	while (curr)
	{
		DequeNode* tmp = curr->nxt;
		deque_node_destroy(tmp);
		free(tmp);
		curr = tmp;
	}
	return true;
}

void deque_node_create(DequeNode* n, void* data)
{
	n->data = data;
	n->nxt	= NULL;
	n->prv	= NULL;
}

void deque_node_destroy(DequeNode* n __attribute__((unused))) { }

bool deque_append(Deque* dq, void* val)
{
	DequeNode* dqn = malloc(sizeof(DequeNode));
	deque_node_create(dqn, val);
	if (!dq->fst)
		dq->fst = dqn;
	if (dq->lst)
		dq->lst->nxt = dqn;
	dqn->prv = dq->lst;
	dqn->nxt = NULL;
	dq->lst	 = dqn;
	return true;
}

bool deque_prepend(Deque* dq, void* val)
{
	DequeNode* dqn = malloc(sizeof(DequeNode));
	deque_node_create(dqn, val);
	if (!dq->lst)
		dq->lst = dqn;
	if (dq->fst)
		dq->fst->prv = dqn;
	dqn->nxt = dq->fst;
	dqn->prv = NULL;
	dq->fst	 = dqn;
	return true;
}

bool deque_empty(Deque* dq) { return !(dq->fst && dq->lst); }

Deque deque_fmap(Deque* dq, void* (*fcn)(void*))
{
	Deque ret;
	deque_create(&ret);
	DequeNode* ncur = NULL;
	DequeNode* ocur = dq->fst;
	DequeNode* prv	= NULL;

	while (ocur)
	{
		ncur = malloc(sizeof(DequeNode));

		if (!ret.fst)
			ret.fst = ncur;

		deque_node_create(ncur, fcn(ocur->data));
		ncur->prv = prv;
		prv->nxt  = ncur;
		prv		  = ncur;
		ocur	  = ocur->nxt;
	}

	ret.lst = ncur;

	return ret;
}

void deque_ifmap(Deque* dq, void* (*fcn)(void*))
{
	DequeNode* cur = dq->fst;
	do
	{
		cur->data = fcn(cur->data);
	} while ((cur = cur->nxt));
}

Deque deque_filter(Deque* dq, bool (*fcn)(void*))
{
	Deque ret;
	deque_create(&ret);
	DequeNode* ncur = NULL;
	DequeNode* ocur = dq->fst;
	DequeNode* prv	= NULL;

	while (ocur)
		if (fcn(ocur->data))
		{
			ncur = malloc(sizeof(DequeNode));
			deque_node_create(ncur, ocur->data);
			ncur->prv = prv;
			prv->nxt  = ncur;
			ocur	  = ocur->nxt;
		}

	ret.lst = ncur;

	return ret;
}

void* deque_foldr(Deque* dq, void* (*op)(void*, void*), void* initial)
{
	DequeNode* cur = dq->fst;
	while (cur)
	{
		initial = op(initial, cur->data);
		cur		= cur->nxt;
	}
	return initial;
}

void* deque_foldl(Deque* dq, void* (*op)(void*, void*), void* initial)
{
	DequeNode* cur = dq->lst;
	while (cur)
	{
		initial = op(initial, cur->data);
		cur		= cur->prv;
	}
	return initial;
}
