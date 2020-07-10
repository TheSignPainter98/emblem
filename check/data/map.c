#include "../../src/data/map.h"

#include <criterion/criterion.h>

Test(map_checks, memory_cycle)
{
	Map m;
	bool rc = map_create(&m);
	cr_assert(rc, "Map was not successfully created\n");
	map_destroy(&m);
}

Test(map_checks, sized_creation)
{
	Map m;
	const size_t s = 10;
	bool rc = map_create_sized(&m, s);
	cr_assert(rc, "Map was not successfully created\n");
	cr_assert(m.tblSize == s, "Map was not of the correct size, expected %ld but got %ld\n", s, m.tblSize);
	map_destroy(&m);
}

Test(map_checks, insertion)
{
	Map m;
	map_create(&m);
	bool rc = map_insert(&m, "some_key", (void*)"asdf");
	cr_assert(rc, "Reported failure to add item to map\n");
	map_destroy(&m);
}

Test(map_checks, removal)
{
	Map m;
	map_create(&m);
	bool rc = map_remove(&m, "some_key");
	cr_assert_not(rc, "Reported successful removal from empty map\n");
	map_insert(&m, "some_key", (void*)"asdf");
	rc = map_remove(&m, "some_key");
	cr_assert(rc, "Reported that an element present in a map couldn't be removed\n");
	map_destroy(&m);
}

Test(map_checks, is_empty)
{
	Map m;
	map_create(&m);
	map_insert(&m, "some_key", (void*)"asdf");
	cr_assert(map_is_empty(&m), "Empty map reported as nonempty\n");
	map_remove(&m, "some_key");
	cr_assert_not(map_is_empty(&m), "Nonempty map reported as empty\n");
	map_destroy(&m);
}

Test(map_checks, has_key)
{
	Map m;
	map_create(&m);
	cr_assert_not(map_has_key(&m, "some_key"), "Map reported having a key when it was empty\n");
	map_insert(&m, "some_key", (void*)"fdsa");
	cr_assert(map_has_key(&m, "some_key"), "Map reported not having a key which was just added\n");
	map_destroy(&m);
}
