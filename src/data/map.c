#include "map.h"

#include "pp/not_implemented.h"
#include <stdio.h>

/**
 * @brief Initial size of a map table
 *
 * @return The initial size of a map table
 */
#define MAP_INITIAL_SIZE		 100
/**
 * @brief The proportion of values stored to the total number of buckets which must be exceeded to cause a resize
 *
 * @return The resizing threshold proprtion
 */
#define MAP_RESIZE_THRESHOLD	 0.5
/**
 * @brief The factor by which the size of a map's table is increased when resizing
 *
 * @return The resize increase factor
 */
#define MAP_SIZE_INCREASE_FACTOR 1.6
/**
 * @brief Key comparator function between a kv pair and a specific key
 *
 * @param kcmp Location of the key comparator function
 *
 * @return A lambda function which compares its input `v1` to the key of its input `v2`
 */
#define pkcmp(kcmp)				 lambda(Cmp, (void* v1, void* v2), kcmp(v1, ((Pair*)v2)->p0))

/**
 * @brief Find the next non-empty bucket with an index strictly after a specified index
 *
 * @param map Map to search
 * @param curr Strict index lower bound
 *
 * @return `true` iff a non-empty bucket at index > `curr` was found, else `false`
 */
static bool next_non_empty_bucket(Map* map, unsigned int* curr);

bool make_map(Map* map, Hasher hash, Comparator kcmp, Destructor ked)
{
	map->curr_stored = 0;
	map->hash		 = hash;
	map->kcmp		 = kcmp;
	map->tbl_size	 = MAP_INITIAL_SIZE;
	map->tbl		 = calloc(map->tbl_size, sizeof(List*));
	map->ked		 = ked;

	return !!map->tbl;
}

void dest_map(Map* map, Destructor ved)
{
	NON_ISO(Destructor dest_kv_pair = ilambda(void, (void* v), {
		Pair* p = v;
		if (p)
		{
			if (map->ked)
				map->ked(p->p0);
			if (ved)
				ved(p->p1);
			free(p);
			fprintf(stderr, "Freed %p\n", v);
		}
	}));
	for (unsigned int i = 0; i < map->tbl_size; i++)
		if (map->tbl[i])
		{
			dest_list(map->tbl[i], true, map->ked || ved ? dest_kv_pair : free);
			free(map->tbl[i]);
		}
	free(map->tbl);
}

bool make_map_from_list(Map* map, List* list, Hasher hash, Comparator kcmp, Destructor ked)
{
	map->curr_stored = 0; // This is updated by the loop's push_map call
	map->hash		 = hash;
	map->kcmp		 = kcmp;
	map->tbl_size	 = MAP_SIZE_INCREASE_FACTOR * list->cnt;
	map->tbl		 = calloc(map->tbl_size, sizeof(List*));
	map->ked		 = ked;

	ListIter iter;
	make_list_iter(&iter, list);
	Pair* val;
	while (iter_list((void**)&val, &iter))
	{
		Maybe _;
		if (!push_map(&_, map, val->p0, val->p1))
			return false;
		dest_maybe(&_, NULL);
	}

	return !!map->tbl;
}

bool push_map(Maybe* oldval, Map* m, void* k, void* v)
{
	// Increase table size of necessary
	size_t resize_threshold = MAP_RESIZE_THRESHOLD * m->tbl_size;
	if (m->curr_stored >= resize_threshold)
	{
		size_t ntbl_size = MAP_SIZE_INCREASE_FACTOR * m->tbl_size;
		List** ntbl = calloc(ntbl_size, sizeof(List*));
		if (!ntbl)
			return false;
		for (unsigned int i = 0; i < m->tbl_size; i++)
			if (m->tbl[i])
			{
				ListIter iter;
				make_list_iter(&iter, m->tbl[i]);
				Pair* kv;
				while (iter_list((void**)&kv, &iter))
				{
					Hash nhb = m->hash(kv->p0) % ntbl_size;
					if (!ntbl[nhb])
					{
						ntbl[nhb] = malloc(sizeof(List));
						if (!ntbl[nhb])
							return false;
						make_list(ntbl[nhb]);
					}
					ListNode* nln = malloc(sizeof(ListNode));
					if (!nln)
						return false;
					make_list_node(nln, kv);
					append_list_node(ntbl[nhb], nln);
				}
				dest_list_iter(&iter);
				dest_list(m->tbl[i], true, NULL);
				free(m->tbl[i]);
			}
		m->tbl_size = ntbl_size;
		free(m->tbl);
		m->tbl = ntbl;
	}

	Hash h = m->hash(k);
	int bh = h % m->tbl_size;
	if (m->tbl[bh])
	{
		Maybe r;
		NON_ISO(in_list_eq(&r, m->tbl[bh], pkcmp(m->kcmp), k));
		switch (r.type)
		{
			case NOTHING:
				make_maybe_nothing(oldval);
				break;
			case JUST:
			{
				ListNode* ln = (ListNode*)r.just;
				make_maybe_just(oldval, ((Pair*)ln->data)->p1);
				remove_list_node(m->tbl[bh], ln);
				if (m->ked)
					m->ked(((Pair*)ln->data)->p0);
				free(ln->data);
				free(ln);
				break;
			}
			default:
				fprintf(stderr, "Unknown data constructor %d\n", r.type);
				exit(0);
		}
	}
	else
	{
		m->tbl[bh] = malloc(sizeof(List));
		if (!m->tbl[bh])
			return false;
		make_list(m->tbl[bh]);
		make_maybe_nothing(oldval);
	}

	Pair* kv	 = malloc(sizeof(Pair));
	if (!kv)
		return false;
	kv->p0		 = k;
	kv->p1		 = v;
	ListNode* nn = malloc(sizeof(ListNode));
	if (!nn)
		return false;
	make_list_node(nn, kv);
	prepend_list_node(m->tbl[bh], nn);

	if (oldval->type == NOTHING)
		m->curr_stored++;

	return true;
}

void get_map(Maybe* mo, Map* map, void* key)
{
	Hash h = map->hash(key);
	int bh = h % map->tbl_size;
	if (!map->tbl[bh])
		make_maybe_nothing(mo);
	else
	{
		Maybe ml;
		NON_ISO(in_list_eq(&ml, map->tbl[bh], pkcmp(map->kcmp), key));
		switch (ml.type)
		{
			case NOTHING:
				make_maybe_nothing(mo);
				break;
			case JUST:
				make_maybe_just(mo, ((Pair*)((ListNode*)ml.just)->data)->p1);
				break;
			default:
				fprintf(stderr, "Unrecognised data constructor: %d\n", ml.type);
				exit(1);
		}

		dest_maybe(&ml, NULL);
	}
}

void make_map_iter(MapIter* iter, Map* map)
{
	iter->bucket_idx = -1;
	iter->map = map;
	if (next_non_empty_bucket(map, &iter->bucket_idx))
	{
		iter->bucket_iter = malloc(sizeof(ListIter));
		make_list_iter(iter->bucket_iter, map->tbl[iter->bucket_idx]);
	}
	else
		iter->bucket_iter = NULL;
}

void dest_map_iter(MapIter* iter)
{
	if (iter->bucket_iter)
	{
		dest_list_iter(iter->bucket_iter);
		free(iter->bucket_iter);
	}
}

bool iter_map(Pair** k, MapIter* iter)
{
	if (!iter->bucket_iter)
		return false;

	while (!iter_list((void**)k, iter->bucket_iter))
	{
		if (!next_non_empty_bucket(iter->map, &iter->bucket_idx))
			return false;
		dest_list_iter(iter->bucket_iter);
		make_list_iter(iter->bucket_iter, iter->map->tbl[iter->bucket_idx]);
	}
	return true;
}

static bool next_non_empty_bucket(Map* map, unsigned int* curr)
{
	for ((*curr)++; *curr < map->tbl_size; (*curr)++)
	{
		if (map->tbl[*curr])
			break;
	}
	return *curr < map->tbl_size;
}

bool iter_map_keys(void** k, MapIter* iter)
{
	Pair* p;
	bool rc = iter_map(&p, iter);
	if (rc)
		*k = p->p0;
	return rc;
}

bool iter_map_values(void** v, MapIter* iter)
{
	Pair* p;
	bool rc = iter_map(&p, iter);
	if (rc)
		*v = p->p1;
	return rc;
}

// TODO: Find the memory leak in map creation
