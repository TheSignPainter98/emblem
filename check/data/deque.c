#include "../../src/data/list.h"

#include <criterion/criterion.h>

static void* incr(void* p);
static void* id(void* a);

Test(list_checks, deque_memory_cycle)
{
	// Create a deque
	Deque dq;
	deque_create(&dq);
	cr_assert(!dq.fst, "The fst field of a fresh, empty deque was not NULL\n");
	cr_assert(!dq.lst, "The lst field of a fresh, empty deque was not NULL\n");

	// Destroy
	deque_destroy(&dq);
}

Test(list_checks, deque_append)
{
	// Create deque
	Deque dq;
	deque_create(&dq);

	// Create and append two nodes
	DequeNode dqn1, dqn2;
	void* d1 = (void*)0xbeefcafe;
	void* d2 = (void*)0xfadedace;
	deque_append(&dq, d1);
	cr_assert(dq.fst->data == d1 && dq.lst->data == d1,
		"Single-item deque first and last pointers didn't point to the unique element\n");
	deque_append(&dq, d2);

	// Check order
	DequeNode* cur = dq.fst;
	cr_assert(cur->data == d1, "First item in deque had wrong value, expected %x but got %x\n", d1, cur->data);
	cr_assert(cur->nxt == &dqn2, "First item added (%p) didn't point to the second (%p) as its next, got %p\n",
		(void*)&dqn1, (void*)&dqn2, (void*)cur->nxt);
	cur = cur->nxt;
	cr_assert(cur->data == d2, "Second item in deque had wrong value, expected %x but got %x\n", d2, cur->data);
	cr_assert(cur == dq.lst, "Second of two nodes added to deque was not the last\n");

	// Destroy
	deque_destroy(&dq);
}

Test(list_checks, deque_prepend)
{
	// Create deque
	Deque dq;
	deque_create(&dq);

	// Create and append two nodes
	DequeNode dqn1, dqn2;
	void* d1 = (void*)0xbeefcafe;
	void* d2 = (void*)0xfadedace;
	deque_prepend(&dq, d1);
	cr_assert(dq.fst->data == d1 && dq.lst->data == d1,
		"Single-item deque first and last pointers didn't point to the unique element\n");
	deque_prepend(&dq, d2);

	// Check order
	DequeNode* cur = dq.fst;
	cr_assert(cur->data == d1, "First item in deque had wrong value, expected %x but got %x\n", d1, cur->data);
	cr_assert(cur->nxt == &dqn2, "First item added (%p) didn't point to the second (%p) as its next, got %p\n",
		(void*)&dqn1, (void*)&dqn2, (void*)cur->nxt);
	cur = cur->nxt;
	cr_assert(cur->data == d2, "Second item in deque had wrong value, expected %x but got %x\n", d2, cur->data);
	cr_assert(cur == dq.lst, "Second of two nodes added to deque was not the last\n");
	// Destroy
}

Test(list_checks, deque_empty)
{
	// Create a deque
	Deque dq;
	deque_create(&dq);

	// Create and add node
	cr_assert(deque_empty(&dq), "Empty deque believed to be non-empty\n");
	void* data = (void*)0xdeadbeef;
	deque_append(&dq, data);
	cr_assert_not(deque_empty(&dq), "Non-empty deque believed to be empry\n");

	// Destroy
	deque_destroy(&dq);
}

Test(list_checks, fmap_id)
{
	Deque dq;
	deque_create(&dq);
	const int numItems = 10;

	for (size_t i = 0; i < numItems; i++)
	{
		void* data = (void*)i;
		deque_append(&dq, data);
	}

	Deque dq2 = deque_fmap(&dq, id);

	DequeNode* cur = dq2.fst;
	for (size_t i = 0; i < numItems; i++)
	{
		cr_assert(cur, "Encountered empty element when testing fmap\n");
		cr_assert((size_t)cur->data == i,
			"Failed to preserve value when id was passed to fmap, expected %ld but got %ld\n", i, (size_t)cur->data);
		DequeNode* nxt = cur->nxt;
		cur = nxt;
	}

	deque_destroy(&dq);
	deque_destroy(&dq2);
}


Test(list_checks, ifmap_id)
{
	Deque dq;
	deque_create(&dq);
	const int numItems = 10;

	for (size_t i = 0; i < numItems; i++)
	{
		void* data = (void*)i;
		deque_append(&dq, data);
	}

	deque_ifmap(&dq, id);

	DequeNode* cur = dq.fst;
	for (size_t i = 0; i < numItems; i++)
	{
		cr_assert(cur, "Encountered empty element when testing fmap\n");
		cr_assert((size_t)cur->data == i,
			"Failed to preserve value when id was passed to fmap, expected %ld but got %ld\n", i, (size_t)cur->data);
		DequeNode* nxt = cur->nxt;
		free((void*)cur->data);
		cur = nxt;
	}

	deque_destroy(&dq);
}

void* id(void* a) { return a; }

Test(list_checks, fmap_incr)
{
	Deque dq;
	deque_create(&dq);
	const int numItems = 10;

	for (size_t i = 0; i < numItems; i++)
	{
		void* data = (void*)i;
		deque_append(&dq, data);
	}

	deque_fmap(&dq, incr);

	DequeNode* cur = dq.fst;
	for (size_t i = 0; i < numItems; i++)
	{
		cr_assert(cur, "Encountered empty element when testing fmap\n");
		cr_assert((size_t)cur->data == i + 1, "Failed to correctly increment data with fmap, expected %ld but for %ld",
			i + 1, cur->data);
		DequeNode* nxt = cur->nxt;
		cur = nxt;
	}

	deque_destroy(&dq);
}

void* incr(void* p) { return (void*)((size_t)p + 1); }
