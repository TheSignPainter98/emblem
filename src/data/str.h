/**
 * @file str.h
 * @brief Exposes functions to hendle the string data-type
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "array.h"
#include "dest-free.h"
#include "maybe.h"
#include <libwapcaplet/libwapcaplet.h>
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
	const char* const str;
	/**
	 * @brief Length of the stored string (does not include the null-terminator)
	 */
	size_t const len;
	/**
	 * @brief Indicates whether memory will be freed
	 */
	bool const free_mem;
	/**
	 * @brief The libwapcaplet internalisation of the string
	 */
	lwc_string* lwc_rep;
} Str;

/**
 * @brief Make a string by reference to a raw value.
 *
 * Does not free stored memory at destruction, assumes this is handled externally for `raw`
 *
 * @param str Pointer to the string to make
 * @param raw Pointer to the raw characters
 */
void make_strv(Str* str, const char* raw);

/**
 * @brief Make a string by reference to a raw value, freeing the raw value when destroyed.
 *
 * @param str Pointer to the string to make
 * @param raw Pointer to the raw characters
 */
void make_strr(Str* str, const char* raw);

/**
 * @brief Make a string by copying another
 *
 * Frees stored memory at destruction
 *
 * @param str Pointer to the string to make
 * @param raw Pointer to the raw characters to copy
 */
void make_strc(Str* str, const char* raw);

/**
 * @brief Destroy a string and free its memory if required
 *
 * @param str Pointer to the string to destroy
 */
void dest_str(Str* str);

/**
 * @brief Destroy and free a string
 *
 * @param str String to destroy
 */
dest_free_sig(str, Str);

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
 * @param ret Pointer to the return value, creates a Maybe object of constructor JUST iff the index was valid. In this
 * case, `ret->just == str[i]`
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
 * @brief Copy the value of a string into another string starting at a given index. Only mutates if the entire string
 * will fit
 *
 * @param cont The container string to write into
 * @param ins The insertion string to read from
 * @param startIdx The index to start writing from
 *
 * @return `true` iff `startIdx` is low enough to allow all of `ins` to fit in `cont` otherwise `false`
 */
bool copy_into_str(Str* cont, Str* ins, size_t startIdx);

/**
 * @brief Duplicate a given string
 *
 * @param o Location to fill with the duplicated string
 * @param todup String to duplicate
 */
void dup_str(Str* o, Str* todup);

/**
 * @brief Obtain the libwapcaplet string version of a string
 *
 * @param si String to internalise/return existing `lwc_string`
 *
 * @return The lwc string equal to si
 */
lwc_string* get_lwc_string(Str* s);
