#pragma once

#include <stdbool.h>
#include <stdlib.h>

#include "types/maybe.h"

typedef struct
{
	void** data;
	size_t cnt;
} Array;

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
 */
void dest_arr(Array* arr);

/**
 * @brief Get array value at a given index
 *
 * Incides which will return a successful value are in the range 0 to the length of the array - 1
 *
 * @param ret Pointer to a Maybe type to populate with the result
 * @param arr Pointer to the array containing the required value
 * @param idx Index of the array element to find
 *
 * @return A Maybe object which contains Just the value at the correct index or Nothing if the index was out of the
 * bounds of the array
 */
void get_arrv(Maybe* ret, Array* arr, size_t idx);

/**
 * @brief Set the value of an array at a particular index
 *
 * @param arr Pointer to the array to modify
 * @param idx Index of the value to change
 * @param val Value to assign to arr[idx]
 *
 * @return Returns 0 iff `idx` is a valid index of `arr`
 */
bool set_arrv(Array* arr, size_t idx, void* val);
