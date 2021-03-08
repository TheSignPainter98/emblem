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
	unsigned int tbl_size;
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

bool make_map(Map* map, Hasher hash, Comparator kcmp, Destructor ked);

void dest_map(Map* map, Destructor ved);

bool make_map_from_list(Map* map, List* list, Hasher hash, Comparator kcmp, Destructor ked);

void push_map(Maybe* oldval, Map* m, void* k, void* v);

void get_map(Maybe* mo, Map* map, void* key);

void make_map_iter(MapIter* iter, Map* map);

void dest_map_iter(MapIter* iter);

bool iter_map(Pair** p, MapIter* iter);

bool iter_map_keys(void** k, MapIter* iter);

bool iter_map_values(void** v, MapIter* iter);
