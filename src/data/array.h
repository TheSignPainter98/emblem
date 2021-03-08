#pragma once

#include "maybe.h"
#include "pp/lambda.h"
#include <stdbool.h>
#include <stdlib.h>

typedef struct
{
	void** data;
	size_t cnt;
} Array;

typedef struct
{
	Array* arr;
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

void make_arr_iter(ArrayIter* iter, Array* arr);
void dest_arr_iter(ArrayIter* iter, Array* arr);
void iter_arr(ArrayIter* iter, Array* arr);
