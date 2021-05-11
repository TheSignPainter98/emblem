#pragma once

#include "list.h"
#include "array.h"

/**
 * @brief Create a list from an array. List myst be freed
 *
 * @param l Pointer to the list to create
 * @param arr Pointer to the array to copy
 */
void make_list_from_arr(List* l, Array* arr);

/**
 * @brief Create an array from a list
 *
 * @param arr Array to write to
 * @param l List ot read
 */
void make_arr_from_list(Array* arr, List* l);
