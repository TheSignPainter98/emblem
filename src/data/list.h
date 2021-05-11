#pragma once

#include "cmp.h"
#include "destructor.h"
#include "maybe.h"
#include "pp/lambda.h"
#include <stdbool.h>
#include <stddef.h>

/**
 * @brief Memory structure of the nodes stored in linked lists
 */
typedef struct ListNode_s
{
	/**
	 * @brief Pointer to the next element or NULL if it is at the end of the list
	 */
	struct ListNode_s* nxt;
	/**
	 * @brief Pointer to the previous element or NULL if it is at the beginning of the list
	 */
	struct ListNode_s* prv;
	/**
	 * @brief Pointer to the data stored at this node
	 */
	void* data;
} ListNode;

/**
 * @brief Linked list
 */
typedef struct
{
	/**
	 * @brief Pointer to the first node of the list or NULL if empty
	 */
	ListNode* fst;
	/**
	 * @brief Pointer to the last node of the list or NULL if empty
	 */
	ListNode* lst;
	/**
	 * @brief The number of elements stored in the list
	 */
	size_t cnt;
} List;

/**
 * @brief Iterator over a list structure
 */
typedef struct
{
	/**
	 * @brief Pointer to the next list node to explore or NULL if at final element of the list
	 */
	ListNode* nxt;
} ListIter;

/**
 * @brief Reversed iterator over a list structure
 */
typedef struct
{
	/**
	 * @brief Pointer to the next list node to explore or NULL if at final element of the list
	 */
	ListNode* nxt;
} ReversedListIter;

/**
 * @brief Initialise a list
 *
 * @param l Pointer to the area of memory to initialise
 */
void make_list(List* l);

/**
 * @brief Destroy a list. Does not affect list elements
 *
 * @param l Pointer to the list to destroy.
 * @param freeNodes Iff not false, frees the memory used by the contained ListNodes
 * @param ed Element destructor called on the data field of each ListNode or NULL
 */
void dest_list(List* l, bool freeNodes, Destructor ed);

/**
 * @brief Initialise a list node
 *
 * @param ln Pointer to the memory to initialise
 * @param data The data to store in the node
 */
void make_list_node(ListNode* ln, void* data);

/**
 * @brief Destroy a list node
 *
 * @param ln Pointer to the list node to destroy
 * @param ed Element destructor to be called on the data field of the list node or NULL
 */
void dest_list_node(ListNode* ln, Destructor ed);

/**
 * @brief Return whether a list is empty
 *
 * @param l List to check
 *
 * @return false iff the list is not empty
 */
bool is_empty_list(List* l);

/**
 * @brief Append a list node to a list
 *
 * @param l Pointer to the list to affect
 * @param ln Pointer to the node to add
 *
 * @return true iff successful
 */
bool append_list_node(List* l, ListNode* ln);

/**
 * @brief Add a node to the front of a list
 *
 * @param l Pointer to the list to change
 * @param ln Pointer to the node to add
 *
 * @return true iff successfil
 */
bool prepend_list_node(List* l, ListNode* ln);

/**
 * @brief Remove a node from a list
 *
 * @param l Pointer to the list which contains `ln`
 * @param ln Pointer to the node to remove
 *
 * @return Returns true iff successful
 */
bool remove_list_node(List* l, ListNode* ln);

/**
 * @brief Initialise an iterator for a list
 *
 * @param i Pointer to the iterator to initialise
 * @param l Pointer to the list which the iterator will run over
 */
void make_list_iter(ListIter* i, List* l);

/**
 * @brief Initialise a reverse-iterator for a list
 *
 * @param i Pointer to the iterator to initialise
 * @param l Pointer to the list which the iterator will run over
 */
void make_reversed_list_iter(ReversedListIter* i, List* l);

/**
 * @brief Destroy an iterator
 *
 * @param i Pointer to the iterator to destroy
 */
void dest_list_iter(ListIter* i);

/**
 * @brief Destroy a reversed iterator
 *
 * @param i Pointer to the iterator to destroy
 */
void dest_reversed_list_iter(ReversedListIter* i);

/**
 * @brief Move the iterator to the next element in the list
 *
 * @param v A location where the value at the current point in the list will be written
 * @param i Pointer to the iterator to use
 *
 * @return false if there are no more elements to iterate, true otherwise
 */
bool iter_list(void** v, ListIter* i);

/**
 * @brief Move the iterator to the next element in the list (when iterated backwards)
 *
 * @param val A location where the value at the current point in the list will be written
 * @param i Pointer to the iterator to use
 *
 * @return false if there are no more elements to iterate, true otherwise
 */
bool iter_list_reversed(void** val, ReversedListIter* i);

/**
 * @brief Checks whether a given element is in a list (by reference-equality)
 *
 * @param l Pointer to the list to check
 * @param val Pointer to the value to test
 *
 * @return true iff the value is in the list
 */
bool in_list(List* l, void* val);

/**
 * @brief Tests whether there exists an element in a given list which is equal under some function
 *
 * @param m Maybe container for the value found to be equal to `val` under `cmp`
 * @param l Pointer to the list to check
 * @param cmp Comparator function to check, val is placed into the first argument.
 * @param val The value to test against
 */
void in_list_eq(Maybe* m, List* l, Comparator cmp, void* val);

/**
 * @brief Concatenate a pair of lists into another
 *
 * @param r The list outputted
 * @param l1 The first list to concatenate.
 * @param l2 The second list to concatenate.
 */
void concat_list(List* r, List* l1, List* l2);

/**
 * @brief Return whether all elements of a list of booleans are true
 * Empty list is vacuously true
 *
 * @param l List to check
 *
 * @return true iff all elements of the list are true
 */
bool all_list(List* l);

/**
 * @brief Return whether there is some element in a list of booleans which is true
 * Empty list is vacuously false
 *
 * @param l List to check
 *
 * @return true iff at at least one element of a list is true.
 */
bool any_list(List* l);
