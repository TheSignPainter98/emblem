/**
 * @file list-array-conversions.c
 * @brief Provides conversoin functions between arrays and lists
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "list-array-conversions.h"

void make_list_from_arr(List* l, Array* arr)
{
	make_list(l);
	l->cnt			   = arr->cnt;
	ListNode* prevNode = NULL;
	for (size_t i = 0; i < arr->cnt; i++)
	{
		ListNode* ln = malloc(sizeof(ListNode));
		make_list_node(ln, arr->data[i]);
		ln->list_mem = true;

		if (!i)
			l->fst = ln;
		if (i == l->cnt - 1)
			l->lst = ln;

		if (prevNode)
			prevNode->nxt = ln;
		ln->prv	 = prevNode;
		prevNode = ln;
	}
}

void make_arr_from_list(Array* arr, List* l)
{
	make_arr(arr, l->cnt);
	ListNode* curr = l->fst;
	for (size_t i = 0; i < l->cnt; i++)
	{
		arr->data[i] = curr->data;
		curr		 = curr->nxt;
	}
}
