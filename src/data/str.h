#pragma once

#include "array.h"
#include "maybe.h"
#include <stdbool.h>
#include <stddef.h>

/**
 * @brief A managed-memory string of fixed length but mutable content
 */
typedef struct
{
	/**
	 * @brief Pointer to the null-terminated memory block
	 */
	char* const str;
	/**
	 * @brief Length of the stored string (does not include the null-terminator)
	 */
	size_t const len;
	/**
	 * @brief Indicates whether memory will be freed
	 */
	bool const free_mem;
} Str;

/**
 * @brief Make an empty string
 *
 * @param str Pointer to the String object to initialise
 */
void make_str(Str* str);

/**
 * @brief Make a string by reference to a raw value.
 *
 * Does not free stored memory at destruction, assumes this is handled externally for `raw`
 *
 * @param str Pointer to the string to make
 * @param raw Pointer to the raw characters
 */
void make_strv(Str* str, char* raw);

/**
 * @brief Make a string by copying another
 *
 * Frees stored memory at destruction
 *
 * @param str Pointer to the string to make
 * @param raw Pointer to the raw characters to copy
 */
void make_strc(Str* str, char* raw);

/**
 * @brief Make a string of specified length.
 *
 * All positions initially have value `\0`
 *
 * @param str Pointer to the string to initialise
 * @param len Length of the string to create
 *
 * @return True iff memory was successfully allocated
 */
bool make_strl(Str* str, size_t len);

/**
 * @brief Destroy a string and free its memory if required
 *
 * @param str Pointer to the string to destroy
 */
void dest_str(Str* str);

/**
 * @brief Create an array (of character values) from a string
 *
 * @param arr Pointer to the array to make
 * @param str Pointer to the string whose data will be copied
 */
void str_to_arr(Array* arr, Str* str);

/**
 * @brief Create a string from an array of characters
 *
 * @param str Pointer to the string to create
 * @param arr Pointer to the array to read
 */
void arr_to_str(Str* str, Array* arr);

/**
 * @brief Get the character at a specified index
 *
 * @param ret Pointer to the return value, creates a Maybe object of constructor JUST iff the index was valid. In this case, `ret->just == str[i]`
 * @param str Pointer to the string to search
 * @param idx Index to get if valid
 */
void get_strc(Maybe* ret, Str* str, size_t idx);

/**
 * @brief Set the value of a character in a string at a specified index
 *
 * @param str Pointer to the string to mutate
 * @param idx Index of the mutation
 * @param val Value to write
 *
 * @return `true` if `idx` was valid else `false`
 */
bool set_strc(Str* str, size_t idx, char val);

/**
 * @brief Copy the value of a string into another string starting at a given index. Only mutates if the entire string will fit
 *
 * @param cont The container string to write into
 * @param ins The insertion string to read from
 * @param startIdx The index to start writing from
 *
 * @return `true` iff `startIdx` is low enough to allow all of `ins` to fit in `cont` otherwise `false`
 */
bool copy_into_str(Str* cont, Str* ins, size_t startIdx);
