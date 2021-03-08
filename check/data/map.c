#include "src/data/map.h"

#include "data/tuple.h"
#include <criterion/criterion.h>

#define NUM_MAP_TEST_ENTRIES 1000

Test(map, memory_life_cycle)
{
	Map m;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);
	dest_map(&m, NULL);
}

Test(map, make_from_list)
{
	List l;
	make_list(&l);
	for (size_t i = 0; i < NUM_MAP_TEST_ENTRIES; i++)
	{
		ListNode* ln = malloc(sizeof(ListNode));
		Pair* p = malloc(sizeof(Pair));
		p->p0 = (void*)i;
		p->p1 = (void*)(i * i);
		make_list_node(ln, p);
		append_list_node(&l, ln);
	}

	Map m;
	make_map_from_list(&m, &l, hash_size_t, cmp_size_ts, NULL);
	for (size_t i = 0; i < NUM_MAP_TEST_ENTRIES; i++)
	{
		Maybe r;
		get_map(&r, &m, (void*)i);
		cr_assert(r.type == JUST, "Failed to find key '%ld' which was added to map by list", i);
		cr_assert((size_t)r.just == i * i, "Retrieved wrong value from map, expected '%ld' but got '%ld'", i * i, (size_t)r.just);
		dest_maybe(&r, NULL);
	}

	cr_assert(m.curr_stored == l.cnt, "Map does not report the same number of entries as were entered, expected %d but got %d", NUM_MAP_TEST_ENTRIES, m.curr_stored);

	dest_map(&m, NULL);
	dest_list(&l, true, free);
}

Test(map, push_get_absent)
{
	Map m;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);
	for (size_t i = 0; i < NUM_MAP_TEST_ENTRIES; i++)
	{
		Maybe r;
		push_map(&r, &m, (void*)i, (void*)(i * i));
		cr_assert(r.type == NOTHING, "Pushing absent value seemed to return an old one");

		Maybe r2;
		get_map(&r2, &m, (void*)i);
		cr_assert(r2.type == JUST, "Getting present value didn't return anything");
		cr_assert((size_t)r2.just == i * i, "Value retrieved from map did not match that put in, expected %ld but got %ld", i * i, (size_t)r2.just);

		dest_maybe(&r, NULL);
		dest_maybe(&r2, NULL);
	}
	cr_assert(m.curr_stored == NUM_MAP_TEST_ENTRIES, "Map does not report the same number of entries as were entered, expected %d but got %d", NUM_MAP_TEST_ENTRIES, m.curr_stored);

	for (size_t i = 0; i < NUM_MAP_TEST_ENTRIES; i++)
	{
		Maybe r;
		get_map(&r, &m, (void*)i);
		cr_assert(r.type == JUST, "Value absent after multiple pushes");
		cr_assert((size_t)r.just == i * i, "Map did not return correct value, expected %ld but got %ld", i, (size_t)r.just);
		dest_maybe(&r, NULL);
	}
	dest_map(&m, NULL);
}

Test(map, push_get_present)
{
	Map m;
	size_t val = 104;
	size_t valv = val + 1;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);
	Maybe r0;
	cr_assert(push_map(&r0, &m, (void*)val, (void*)(valv)), "Successfully pushing to map did not return true");
	dest_maybe(&r0, NULL);
	for (size_t i = 0; i < NUM_MAP_TEST_ENTRIES; i++)
	{
		Maybe r;
		cr_assert(push_map(&r, &m, (void*)val, (void*)(val * val)), "Successfully pushing to map did not return true");
		cr_assert(r.type == JUST, "Pushing present value did not return the old one");
		cr_assert((size_t)r.just == (i ? val * val : valv), "Pushing present value did not return original value, expected %ld but got %ld instead", valv, r.just);

		Maybe r2;
		get_map(&r2, &m, (void*)val);
		cr_assert(r2.type == JUST, "Getting present value didn't return anything");
		cr_assert((size_t)r2.just == val * val, "Value retrieved from map did not match that put in");

		dest_maybe(&r, NULL);
		dest_maybe(&r2, NULL);
	}
	cr_assert(m.curr_stored == 1, "Map does not report the same number of entries as were entered, expected %d but got %d", NUM_MAP_TEST_ENTRIES, m.curr_stored);
	dest_map(&m, NULL);
}

Test(map, iter_life_cycle)
{
	Map m;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);

	MapIter iter;
	make_map_iter(&iter, &m);

	dest_map_iter(&iter);
	dest_map(&m, NULL);
}

Test(map, iter)
{
	Map m;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);
	Maybe _;
	push_map(&_, &m, (void*)1, (void*)2);
	push_map(&_, &m, (void*)2, (void*)4);
	push_map(&_, &m, (void*)3, (void*)6);
	push_map(&_, &m, (void*)4, (void*)8);

	MapIter iter;
	make_map_iter(&iter, &m);
	for (size_t i = 1; i <= 4; i++)
	{
		Pair* p;
		cr_assert(iter_map(&p, &iter), "Iter failed to iterate early");
		POINTER_TO_INT_CAST(cr_assert(1 <= (int)p->p0 && (int)p->p0 <= 4, "Iter returned key which was never in the map, expected e ∈ [1,4] but got e = %d", (int)p->p0));
		POINTER_TO_INT_CAST(cr_assert((int)p->p1 == (int)p->p0 << 1, "Iter returned non-matching key-value pair"));
	}

	dest_map_iter(&iter);
	dest_maybe(&_, NULL);
	dest_map(&m, NULL);
}

Test(map, iter_keys)
{
	Map m;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);
	Maybe _;
	push_map(&_, &m, (void*)1, (void*)2);
	push_map(&_, &m, (void*)2, (void*)4);
	push_map(&_, &m, (void*)3, (void*)6);
	push_map(&_, &m, (void*)4, (void*)8);

	MapIter iter;
	make_map_iter(&iter, &m);
	for (size_t i = 1; i <= 4; i++)
	{
		size_t k;
		cr_assert(iter_map_keys((void**)&k, &iter), "Iter failed to iterate early");
		cr_assert(1 <= k && k <= 4, "Value iter did not return a value present in the map, expected one of 2,4,6,8 but got %ld", k);
	}

	dest_map_iter(&iter);
	dest_maybe(&_, NULL);
	dest_map(&m, NULL);
}

Test(map, iter_values)
{
	Map m;
	make_map(&m, hash_size_t, cmp_size_ts, NULL);
	Maybe _;
	push_map(&_, &m, (void*)1, (void*)2);
	push_map(&_, &m, (void*)2, (void*)4);
	push_map(&_, &m, (void*)3, (void*)6);
	push_map(&_, &m, (void*)4, (void*)8);

	MapIter iter;
	make_map_iter(&iter, &m);
	for (size_t i = 1; i <= 4; i++)
	{
		size_t v;
		cr_assert(iter_map_values((void**)&v, &iter), "Iter failed to iterate early");
		cr_assert(2 <= v && v <= 8 && v % 2 == 0, "Value iter did not return a value present in the map, expected one of 2,4,6,8 but got %ld", v);
	}

	dest_map_iter(&iter);
	dest_maybe(&_, NULL);
	dest_map(&m, NULL);
}
