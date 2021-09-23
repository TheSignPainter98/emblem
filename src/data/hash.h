/**
 * @file hash.h
 * @brief Exposes hash functions for standard data types
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include <stddef.h>

/**
 * @brief Type of a hashed object
 */
typedef unsigned int Hash;
/**
 * @brief Type of a function which takes an object and returns its hash
 *
 * @param v A value to hash
 *
 * @return The hash of `v`
 */
typedef Hash (*Hasher)(void* v);

/**
 * @brief Signature of a hashing function
 *
 * @param name The name of the type to hash
 *
 * @return A signature for a function which hashes `name`s
 */
#define HASH_SIG(name) Hash hash_##name(void* v)

/**
 * @brief Hash a character
 *
 * @param v A value to hash
 *
 * @return The hash of `v`
 */
HASH_SIG(char);
/**
 * @brief Hash an int
 *
 * @param v A value to hash
 *
 * @return The hash of `v`
 */
HASH_SIG(int);
/**
 * @brief Hash a size_t
 *
 * @param v A value to hash
 *
 * @return The hash of `v`
 */
HASH_SIG(size_t);
/**
 * @brief Hash a void pointer. This operates on the numerical value of the pointer, and not its content!
 *
 * @param v A value to hash
 *
 * @return The hash of `v`
 */
HASH_SIG(ptr);
/**
 * @brief Hash a string
 *
 * @param v A value to hash
 *
 * @return The hash of `v`
 */
HASH_SIG(str);
