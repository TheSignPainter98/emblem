#include "data/list.h"

#include <criterion/criterion.h>

Cmp weird_eq(void* v1, void* v2);

Test(list, memory_life_cycle)
{
	List l;
	make_list(&l);
	cr_assert_not(l.fst, "List first element has non-NULL value at initialisation");
	cr_assert_not(l.lst, "List last element has non-NULL value at initialisation");
	cr_assert_not(l.cnt, "List count is non-zero at initialisation");
	dest_list(&l, false, NULL);
}

Test(list, node_memory_life_cycle)
{
	ListNode ln;
	long int data = 103L;
	make_list_node(&ln, (void*)data);
	cr_assert_not(ln.nxt, "List node has non-NULL next element at initialisation");
	cr_assert_not(ln.prv, "List node has non-NULL previous element at initialisation");
	cr_assert((long int)ln.data == data, "List node data not the same as that which was input");
	dest_list_node(&ln, NULL);
}

Test(list, append_one_node)
{
	List l;
	make_list(&l);
	ListNode ln;
	long int val = 104L;
	make_list_node(&ln, (void*)val);
	append_list_node(&l, &ln);

	cr_assert(l.fst == &ln, "List first node not does not point to the sole element");
	cr_assert(l.lst == &ln, "List lst node not does not point to the sole element");
	cr_assert(l.cnt == 1, "List of one item did not have unit length");
	cr_assert((long int)l.fst->data == val, "List element value not preserved");

	dest_list_node(&ln, NULL);
	dest_list(&l, false, NULL);
}

Test(list, append_many_nodes)
{
	List l;
	make_list(&l);
	ListNode* ln;
	const size_t lns = 100;
	for (size_t i = 0; i < lns; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)(i * i));
		append_list_node(&l, ln);
		cr_assert(ln->data == (void*)(i * i), "List node value incorrectly set");
	}

	cr_assert(l.cnt == lns,
		"Length of list was not equal to the number of elements within it, expected %ld but got %ld", lns, l.cnt);
	cr_assert(l.fst, "List first element was never set");
	cr_assert(l.lst, "List last element was never set");

	ln = l.fst;
	for (size_t i = 0; i < lns; i++)
	{
		cr_assert(ln, "List element %ld was NULL", i);
		cr_assert((size_t)ln->data == i * i, "List element had incorrect value, expected %ld but got %ld", i * i,
			(size_t)ln->data);
		ln = ln->nxt;
	}

	ln = l.lst;
	for (size_t i = 0; i < lns; i++)
	{
		cr_assert(ln, "List element %ld was NULL", i);
		cr_assert((size_t)ln->data == (lns - i - 1) * (lns - i - 1),
			"List element had incorrect value, expected %ld but got %ld", i * i, (size_t)ln->data);
		ln = ln->prv;
	}

	dest_list(&l, true, NULL);
}

Test(list, append_remove_one_element)
{
	List l;
	make_list(&l);
	ListNode ln;
	make_list_node(&ln, NULL);
	append_list_node(&l, &ln);
	remove_list_node(&l, &ln);
	cr_assert_not(l.fst, "Empty list first element was not null");
	cr_assert_not(l.lst, "Empty list last element was not null");
	cr_assert_not(l.cnt, "Empty list count was not zero");

	dest_list_node(&ln, NULL);
	dest_list(&l, false, NULL);
}

Test(list, append_remove_many_elements)
{
	List l;
	make_list(&l);
	ListNode* ln;
	const size_t lns = 100;
	for (size_t i = 0; i < lns; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)(i * i));
		append_list_node(&l, ln);
		cr_assert(ln->data == (void*)(i * i), "List node value incorrectly set");
	}

	ln = l.fst;
	for (size_t i = 0; i < lns; i++)
	{
		ListNode* lnn = ln->nxt;
		remove_list_node(&l, ln);

		cr_assert(ln, "List node %ld was unexpectedly NULL", i);
		cr_assert(ln->data == (void*)(i * i), "List node data changed by removal");
		cr_assert_not(ln->nxt, "List node next element not NULL after removal");
		cr_assert_not(ln->prv, "List node previous element not NULL after removal");

		dest_list_node(ln, NULL);
		free(ln);
		ln = lnn;
	}

	cr_assert_not(l.cnt, "Length of list was not zero after all elements removed, got %ld", l.cnt);
	cr_assert_not(l.fst, "List with all elements removed still had first element set");
	cr_assert_not(l.lst, "List with all elements removed still had last element set");

	dest_list(&l, true, NULL);
}

Test(list, prepend_one_node)
{
	List l;
	make_list(&l);
	ListNode ln;
	long int val = 104L;
	make_list_node(&ln, (void*)val);
	prepend_list_node(&l, &ln);

	cr_assert(l.fst == &ln, "List first node not does not point to the sole element");
	cr_assert(l.lst == &ln, "List lst node not does not point to the sole element");
	cr_assert(l.cnt == 1, "List of one item did not have unit length");
	cr_assert((long int)l.fst->data == val, "List element value not preserved");

	dest_list_node(&ln, NULL);
	dest_list(&l, false, NULL);
}

Test(list, prepend_many_nodes)
{
	List l;
	make_list(&l);
	ListNode* ln;
	const size_t lns = 100;
	for (size_t i = 0; i < lns; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)(i * i));
		prepend_list_node(&l, ln);
		cr_assert(ln->data == (void*)(i * i), "List node value incorrectly set");
	}

	cr_assert(l.cnt == lns,
		"Length of list was not equal to the number of elements within it, expected %ld but got %ld", lns, l.cnt);
	cr_assert(l.fst, "List first element was never set");
	cr_assert(l.lst, "List last element was never set");

	ln = l.fst;
	for (size_t i = 0; i < lns; i++)
	{
		cr_assert(ln, "List element %ld was NULL", i);
		cr_assert((size_t)ln->data == (lns - i - 1) * (lns - i - 1),
			"List element had incorrect value, expected %ld but got %ld", (lns - i - 1) * (lns - i - 1),
			(size_t)ln->data);
		ln = ln->nxt;
	}

	ln = l.lst;
	for (size_t i = 0; i < lns; i++)
	{
		cr_assert(ln, "List element %ld was NULL", i);
		cr_assert((size_t)ln->data == i * i, "List element had incorrect value, expected %ld but got %ld", i * i,
			(size_t)ln->data);
		ln = ln->prv;
	}

	ln = l.fst;
	while (ln)
	{
		ListNode* lnn = ln->nxt;
		dest_list_node(ln, NULL);
		ln = lnn;
	}

	dest_list(&l, true, NULL);
}

Test(list, prepend_remove_one_element)
{
	List l;
	make_list(&l);
	ListNode ln;
	make_list_node(&ln, NULL);
	prepend_list_node(&l, &ln);
	remove_list_node(&l, &ln);
	cr_assert_not(l.fst, "Empty list first element was not null");
	cr_assert_not(l.lst, "Empty list last element was not null");
	cr_assert_not(l.cnt, "Empty list count was not zero");

	dest_list_node(&ln, NULL);
	dest_list(&l, false, NULL);
}

Test(list, prepend_remove_many_elements)
{
	List l;
	make_list(&l);
	ListNode* ln;
	const size_t lns = 100;
	for (size_t i = 0; i < lns; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)(i * i));
		prepend_list_node(&l, ln);
		cr_assert(ln->data == (void*)(i * i), "List node value incorrectly set");
	}

	ln = l.fst;
	for (size_t i = 0; i < lns; i++)
	{
		ListNode* lnn = ln->nxt;
		remove_list_node(&l, ln);

		cr_assert(ln, "List node %ld was unexpectedly NULL", i);
		cr_assert(ln->data == (void*)((lns - i - 1) * (lns - i - 1)), "List node data changed by removal");
		cr_assert_not(ln->nxt, "List node next element not NULL after removal");
		cr_assert_not(ln->prv, "List node previous element not NULL after removal");

		dest_list_node(ln, NULL);
		free(ln);
		ln = lnn;
	}

	cr_assert_not(l.cnt, "Length of list was not zero after all elements removed, got %ld", l.cnt);
	cr_assert_not(l.fst, "List with all elements removed still had first element set");
	cr_assert_not(l.lst, "List with all elements removed still had last element set");

	dest_list(&l, true, NULL);
}

Test(list, iter_memory_cycle)
{
	List l;
	make_list(&l);
	ListIter i;
	make_list_iter(&i, &l);
	dest_list_iter(&i);
	dest_list(&l, false, NULL);
}

Test(list, iter)
{
	List l;
	make_list(&l);
	ListNode ln;
	long int val = 104L;
	make_list_node(&ln, (void*)val);
	append_list_node(&l, &ln);

	ListIter i;
	make_list_iter(&i, &l);
	cr_assert(i.nxt == l.fst, "List iterator initial current element was not equal to the first element of the list");
	long int sval;
	cr_assert(iter_list((void**)&sval, &i), "Failed to iterate over first element of unitary list");
	cr_assert(val == sval, "Iterator returned value was not equal to the that stored %p %p", val, sval);
	cr_assert(i.nxt == NULL, "Iterator next element was not NULL at end of list");
	cr_assert_not(iter_list((void**)&val, &i), "Iterator could iterate multuple times on unitary list");

	dest_list_node(&ln, NULL);
	dest_list(&l, false, NULL);
}

Test(list, reversed_iter_memory_cycle)
{
	List l;
	make_list(&l);
	ReversedListIter i;
	make_reversed_list_iter(&i, &l);
	dest_reversed_list_iter(&i);
	dest_list(&l, false, NULL);
}

Test(list, is_empty)
{
	List l;
	make_list(&l);

	cr_assert(is_empty_list(&l), "Newly-created list is not empty");

	ListNode ln;
	make_list_node(&ln, (void*)104);
	append_list_node(&l, &ln);

	cr_assert_not(is_empty_list(&l), "Singleton list is considered empty");

	dest_list(&l, false, NULL);
}

Test(list, make_from_array)
{
	const size_t arrlen = 100;
	List l;
	Array arr;
	make_arr(&arr, arrlen);

	for (size_t i = 0; i < arrlen; i++)
		set_arrv(&arr, i, (void*)i);

	make_list_from_arr(&l, &arr);

	cr_assert(
		l.cnt == arr.cnt, "List created from array had different length, expected %ld but got %ld", l.cnt, arr.cnt);

	ListIter iter;
	make_list_iter(&iter, &l);
	void* val;
	size_t i = 0;
	while (iter_list(&val, &iter))
	{
		cr_assert((size_t)val == i, "Incorrect value in list from array, expected %ld but got %ld", i, (size_t)val);
		i++;
	}

	dest_list(&l, true, NULL);
	dest_arr(&arr, NULL);
}

Test(list, in)
{
	const size_t llen = 100;
	List l;
	make_list(&l);

	ListNode* ln;
	for (size_t i = 0; i < llen; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)i);
		append_list_node(&l, ln);
	}

	const size_t needle		   = 51;
	const size_t missingNeedle = 2 * llen;
	bool r					   = in_list(&l, (void*)needle);
	cr_assert(r, "Value %ld was not present in list of the numbers 0..%ld", needle, llen - 1);

	bool r2 = in_list(&l, (void*)missingNeedle);
	cr_assert_not(r2, "Value %ld was present in list of the numbers 0..%ld", missingNeedle, llen - 1);

	dest_list(&l, true, NULL);
}

Test(list, in_eq)
{
	const size_t llen = 100;
	List l;
	make_list(&l);

	ListNode* ln;
	for (size_t i = 0; i < llen; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)i);
		append_list_node(&l, ln);
	}

	Maybe m1;
	Maybe m2;
	Maybe m3;
	Maybe m4;
	const size_t needle				= 51;
	const size_t trickNeedle		= -1;
	const size_t missingNeedle		= 2 * (llen + 1);
	const size_t trickMissingNeedle = llen - 1;

	in_list_eq(&m1, &l, weird_eq, (void*)needle);
	in_list_eq(&m2, &l, weird_eq, (void*)trickNeedle);
	in_list_eq(&m3, &l, weird_eq, (void*)missingNeedle);
	in_list_eq(&m4, &l, weird_eq, (void*)trickMissingNeedle);

	cr_assert(m1.type == JUST, "Value %ld was not present in list of the numbers 0..%ld under eq-condition v1=v2-1",
		needle, llen - 1);
	cr_assert((size_t)((ListNode*)m1.just)->data == needle + 1,
		"Retrieved incorrect value from list under fancy equality, expected %ld but got %ld", needle,
		(size_t)((ListNode*)m1.just)->data);
	cr_assert(m2.type == JUST, "Value %ld was not present in list of the numbers 0..%ld under eq-condition v1=v2-1",
		trickNeedle, llen - 1);
	cr_assert((size_t)((ListNode*)m2.just)->data == trickNeedle + 1,
		"Retrieved incorrect value from list inder fancy equality, expected %ld but got %ld", trickNeedle,
		(size_t)((ListNode*)m2.just)->data);
	cr_assert(m3.type == NOTHING, "Value %ld present in list of the numbers 0..%ld under eq-condition v1=v2-1",
		missingNeedle, llen - 1);
	cr_assert(m4.type == NOTHING, "Value %ld present in list of the numbers 0..%ld under eq-condition v1=v2-1",
		trickMissingNeedle, llen - 1);

	dest_maybe(&m1, NULL);
	dest_maybe(&m2, NULL);
	dest_maybe(&m3, NULL);
	dest_maybe(&m4, NULL);

	dest_list(&l, true, NULL);
}

Cmp weird_eq(void* v1, void* v2)
{
	size_t s1 = (size_t)v1;
	size_t s2 = (size_t)v2 - 1;
	return s1 < s2 ? CMP_LT : s1 == s2 ? CMP_EQ : CMP_GT;
}

Test(list, all)
{
	const size_t llen = 100;
	List l;
	make_list(&l);

	ListNode* ln;
	for (size_t i = 0; i < llen; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)true);
		append_list_node(&l, ln);
	}

	cr_assert(all_list(&l), "List of all true values was not recognised as such");

	l.lst->data = (void*)false;

	cr_assert_not(all_list(&l), "List with false values was considered as all true");

	dest_list(&l, true, NULL);
}

Test(list, any)
{
	const size_t llen = 100;
	List l;
	make_list(&l);

	ListNode* ln;
	for (size_t i = 0; i < llen; i++)
	{
		ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)false);
		append_list_node(&l, ln);
	}

	cr_assert_not(any_list(&l), "List of all false values was not recognised as such");

	l.lst->data = (void*)true;

	cr_assert(any_list(&l), "List with true values was considered as all false");

	dest_list(&l, true, NULL);
}

Test(list, concat)
{
	List l1;
	List l2;
	const int change_point = 50;
	const int lns		   = 100;
	make_list(&l1);
	make_list(&l2);
	for (size_t i = 0; i < lns; i++)
	{
		ListNode* ln = malloc(sizeof(ListNode));
		make_list_node(ln, (void*)i);
		List* l = i < change_point ? &l1 : &l2;
		append_list_node(l, ln);
	}

	List lr;
	concat_list(&lr, &l1, &l2);
	cr_assert(lr.cnt == lns, "Concatenated list length incorrect, expected %d but got %d", lns, lr.cnt);
	cr_assert(lr.fst->data == l1.fst->data, "Concatenated list had different first node");
	cr_assert(lr.lst->data == l2.lst->data, "Concatenated list has incorrect last");

	ListNode* curr = lr.fst;
	for (size_t i = 0; i < lns; i++)
	{
		cr_assert((size_t)curr->data == i,
			"Concatenated list had incorrect stored value (iterated forwards), expected %ld but got %ld", i,
			(size_t)curr->data);

		curr = curr->nxt;
	}

	curr = lr.lst;
	for (ssize_t i = lns - 1; i >= 0; i--)
	{
		cr_assert((size_t)curr->data == (size_t)i,
			"Concatenated list had incorrect stored value (iterated backwards), expected %ld but got %ld", i,
			(size_t)curr->data);

		curr = curr->prv;
	}

	dest_list(&lr, true, NULL);
	dest_list(&l1, true, NULL);
	dest_list(&l2, true, NULL);
}
