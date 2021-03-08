#pragma once

#include "cmp.h"
#include "destructor.h"
#include "pp/lambda.h"
#include <stdbool.h>
#include <stddef.h>

typedef struct ListNode_s
{
	struct ListNode_s* nxt;
	struct ListNode_s* prv;
	void* data;
} ListNode;

typedef struct
{
	ListNode* fst;
	ListNode* lst;
	size_t cnt;
} List;

typedef struct
{
	ListNode* nxt;
} ListIter;

#include "array.h"

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
 * @brief Destroy an iterator
 *
 * @param i Pointer to the iterator to destroy
 */
void dest_list_iter(ListIter* i);

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
 * @brief Create a list from an array. List myst be freed
 *
 * @param l Pointer to the list to create
 * @param arr Pointer to the array to copy
 */
void make_list_from_arr(List* l, Array* arr);

/**
 * @brief Checks whether a given element is in a list (by reference-equality)
 *
 * @param l Pointer to the list to check
 * @param val Pointer to the value to test
 *
 * @return true iff the value is in the list
 */
void in_list(Maybe* m, List* l, void* val);

/**
 * @brief Tests whether there exists an element in a given list which is equal under some function
 *
 * @param l Pointer to the list to check
 * @param eq Function to check
 * @param val The value to test against
 *
 * @return true iff present
 */
void in_list_eq(Maybe* m, List* l, Comparator cmp, void* val);

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
