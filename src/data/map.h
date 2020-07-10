#ifndef MAP_H_
#define MAP_H_

#include "list.h"
#include <stdbool.h>
#include <stddef.h>

#define MAP_INITIAL_TABLE_SIZE 97

typedef struct
{
	const char* key;
	const void* val;
} MapEntry;

typedef struct
{
	const Deque* tbl;
	const size_t tblSize;
	size_t numElements;
} Map;

bool map_create(Map* map);
bool map_create_sized(Map* map, size_t initialSize);
void map_destroy(Map* map);
bool map_is_empty(Map* map);
bool map_has_key(Map* map, const char* key);
bool map_insert(Map* map, const char* key, void* val);
bool map_remove(Map* map, const char* key);

#endif /* MAP_H_ */
