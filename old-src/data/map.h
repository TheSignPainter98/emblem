/**
 * @file map.h
 * @brief Exposes functions to handle the Map data structure
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "cmp.h"
#include "destructor.h"
#include "hash.h"
#include "list.h"
#include "maybe.h"
#include "pp/lambda.h"
#include "tuple.h"

typedef struct
{
	List** tbl;
	size_t tbl_size;
	size_t curr_stored;
	Hasher hash;
	Comparator kcmp;
	Destructor ked;
} Map;

typedef struct
{
	Map* map;
	ListIter* bucket_iter;
	unsigned int bucket_idx;
} MapIter;

/**
 * @brief Make a map
 *
 * @param map Pointer to the map to create
 * @param hash Hash function to use on keys
 * @param kcmp Comparator to use on keys
 * @param ked Destructor to use on keys
 *
 * @return `true` iff sufficient memory could be allocated to the map, `false` otherwise
 */
bool make_map(Map* map, Hasher hash, Comparator kcmp, Destructor ked);

/**
 * @brief Destroy a map
 *
 * @param map Pointer to the map to destroy
 * @param ved Value destructor
 */
void dest_map(Map* map, Destructor ved);

/**
 * @brief Make a map from a list of key-value Pairs
 *
 * @param map Pointer to the map to create
 * @param list Pointer to the list of key-value Pairs
 * @param hash Hash function to use on keys
 * @param kcmp Comparison function to use on keys
 * @param ked Destructor for the keys
 *
 * @return `true` iff enough memory could be allocated to create the map, `false` otherwise
 */
bool make_map_from_list(Map* map, List* list, Hasher hash, Comparator kcmp, Destructor ked);

/**
 * @brief Push a value into a map
 *
 * @param oldval Pointer to a maybe type to be populated with the old value at a given key, if present
 * @param m Pointer to the map to populate
 * @param k Key to push
 * @param v Value to push
 *
 * @return `true` iff sufficient memory was available during pushing and (if necessary) resizing, otherwise `false`
 */
bool push_map(Maybe* oldval, Map* m, void* k, void* v);

/**
 * @brief Get a value from a map
 *
 * @param mo Return value populated with the value if present
 * @param map Pointer to the map to retrieve the value from
 * @param key Key to use
 */
void get_map(Maybe* mo, Map* map, void* key);

/**
 * @brief Make an iterator over the map
 *
 * @param iter Pointer to the iterator to make
 * @param map Pointer to the map to iterate over
 */
void make_map_iter(MapIter* iter, Map* map);

/**
 * @brief Destroy a map iterator
 *
 * @param iter Pointer to the iterator to destroy
 */
void dest_map_iter(MapIter* iter);

/**
 * @brief Iterate once over key-value pairs using a specified iterator
 *
 * @param p Pointer to the return pair
 * @param iter Pointer to the iterator to use
 *
 * @return `true` if iteration was successful, otherwise `false` as iteration has finished
 */
bool iter_map(Pair** p, MapIter* iter);

/**
 * @brief Iterate once over the keys in a map using a given iterator
 *
 * @param k Pointer to the key to return
 * @param iter Pointer to the iterator to use
 *
 * @return `true` iff iteration was successful, otherwise `false` as iteration has finished
 */
bool iter_map_keys(void** k, MapIter* iter);

/**
 * @brief Iterate over the values in a map using a given iterator
 *
 * @param v Pointer to the value to return
 * @param iter Pointer to the iterator to use
 *
 * @return `true` iff iteration was successful, otherwise `false` as iteration has finished
 */
bool iter_map_values(void** v, MapIter* iter);
