#pragma once

#include "maybe.h"
#include "pp/lambda.h"
#include <stdbool.h>
#include <stdlib.h>

/**
 * @brief Managed-memory array structure with safe access
 */
typedef struct
{
	/**
	 * @brief Pointer to the data stored
	 */
	void** data;
	/**
	 * @brief Number of elements in the stored array
	 */
	size_t cnt;
} Array;

/**
 * @brief Array iterator
 */
typedef struct
{
	/**
	 * @brief Pointer to the array to iterate over
	 */
	Array* arr;
	/**
	 * @brief Index of the next element to iterate over
	 */
	size_t next;
} ArrayIter;

#include "list.h"

/**
 * @brief Make an array object, a block of memory of finite length
 *
 * @param arr Pointer to the array to initialise.
 * @param cnt Length of the array
 *
 * @return True iff memory allocation was successful
 */
bool make_arr(Array* arr, size_t cnt);

/**
 * @brief Destroy an array object
 *
 * @param arr Pointer to the array to destroy
 * @param ed  Element destructor function or NULL
 */
void dest_arr(Array* arr, func_sig(void, ed, (void*)));

/**
 * @brief Get array value at a given index
 *
 * Incides which will return a successful value are in the range 0 to the length of the array - 1
 *
 * @param ret Pointer to a Maybe type to populate with the result
 * @param arr Pointer to the array containing the required value
 * @param idx Index of the array element to find
 */
void get_arrv(Maybe* ret, Array* arr, size_t idx);

/**
 * @brief Set the value of an array at a particular index
 *
 * @param arr Pointer to the array to modify
 * @param idx Index of the value to change
 * @param val Value to assign to arr[idx]
 *
 * @return Returns true iff `idx` is a valid index of `arr`
 */
bool set_arrv(Array* arr, size_t idx, void* val);

/**
 * @brief Create an array from a list
 *
 * @param arr Array to write to
 * @param l List ot read
 */
void make_arr_from_list(Array* arr, List* l);

/**
 * @brief Make an array iterator
 *
 * @param iter Pointer to the iterator to make
 * @param arr Pointer to the array to iterate over
 */
void make_arr_iter(ArrayIter* iter, Array* arr);

/**
 * @brief Destror an array iterator
 *
 * @param iter Pointer to the iterator to destroy
 */
void dest_arr_iter(ArrayIter* iter);

/**
 * @brief Iterate once over an array
 *
 * @param v Pointer to the return value
 * @param iter Pointer to the array iterator to use
 *
 * @return True iff a value was successfully written to `v`, false iff there were no elements left
 */
bool iter_arr(void** v, ArrayIter* iter);
