#include "map.h"

#include "../logs/logs.h"
#include <stdlib.h>
#include <string.h>

static bool map_entry_destroy(MapEntry* e);
static size_t str_hash(const char* str);
static void* map_entry_destroy_wrapper(void* me);

bool map_create(Map* map) { return map_create_sized(map, MAP_INITIAL_TABLE_SIZE); }

bool map_create_sized(Map* map, size_t initialSize)
{
	map->numElements		= 0;
	*(size_t*)&map->tblSize = initialSize;
	map->tbl				= calloc(map->tblSize, sizeof(MapEntry));
	if (!map->tbl)
	{
		log_err("Failed to allocate sufficient space for map\n");
		return false;
	}
	return true;
}

void map_destroy(Map* map)
{
	for (size_t i = 0; i < map->tblSize; i++)
	{
		Deque* d = (Deque*)&map->tbl[i];
		deque_fmap(d, map_entry_destroy_wrapper);
		deque_destroy(d);
	}

	free((void*)map->tbl);
}

void* map_entry_destroy_wrapper(void* me) { return (void*)map_entry_destroy((MapEntry*)me); }
bool map_entry_create(MapEntry* e, const char* key, const void* val)
{
	e->key = strdup(key);
	e->val = val;
	return !e->key;
}

bool map_entry_destroy(MapEntry* e)
{
	free((char*)e->key);
	return true;
}

bool map_is_empty(Map* map) { return !map->numElements; }

bool map_has_key(Map* map, const char* key)
{
	const Deque d = map->tbl[str_hash(key)];
	DequeNode* n  = d.fst;
	do
		if (!strcmp(((MapEntry*)n->data)->key, key))
			return true;
	while ((n = n->nxt));
	return false;
}

size_t str_hash(const char* str)
{
	// Using Dan Bernstein's dbl2 algorithm
	size_t hash = 5381;
	int c;
	while ((c = *str++))
		hash = ((hash << 5) + hash) + c;
	return hash;
}

bool map_insert(
	Map* map __attribute__((unused)), const char* key __attribute__((unused)), void* val __attribute__((unused)))
{

	return false;
}

bool map_remove(Map* map __attribute__((unused)), const char* key __attribute__((unused))) { return false; }
