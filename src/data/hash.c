/**
 * @file hash.c
 * @brief Provides hash functions for standard data types
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "hash.h"

#include "pp/assert.h"
#include "pp/ignore_warning.h"
#include "str.h"
#include <limits.h>

/**
 * @brief Construct a function which hashes a specified numeric type
 *
 * @param name The name of the type to hash
 * @param type The type of the values to hash
 *
 * @return A function which hashes `type`s
 */
#define HASH_NUM(name, type)                                                                                           \
	HASH_SIG(name)                                                                                                     \
	{                                                                                                                  \
		unsigned long int hash = 0xdcba0987654321;                                                                     \
		for (unsigned int i = 0; i < CHAR_BIT * sizeof(Hash); i++)                                                     \
		{                                                                                                              \
			POINTER_TO_INT_CAST(hash ^= (type)v << i);                                                                 \
		}                                                                                                              \
		return hash;                                                                                                   \
	}

HASH_NUM(char, char)
HASH_NUM(int, int)
HASH_NUM(size_t, size_t)

HASH_SIG(ptr)
{
	ASSERT(sizeof(void*) == sizeof(size_t));
	return hash_size_t(v);
}

#define DJB2_INITIAL_HASH 5381
#define DJB2_SHIFT		  5
HASH_SIG(str)
{
	// The djb2 algorithm
	Hash h = DJB2_INITIAL_HASH;
	char c;
	char* str = ((Str*)v)->str;
	while ((c = *str++))
		h = ((h << DJB2_SHIFT) + h) ^ c;

	return h;
}
