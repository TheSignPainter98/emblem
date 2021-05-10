#include "src/data/locked.h"

#include <criterion/criterion.h>

Test(locked, memory_cycle)
{
	Locked l;
	char data[] = "Hello, world";
	make_locked(&l, data);
	dest_locked(&l, NULL);
}

Test(locked, mutex_and_return)
{
	Locked l;
	char data[] = "Hello, world";
	make_locked(&l, data);

	char* locked_data = lock(&l);
	cr_assert(locked_data == data, "Lock did not return the correct data, expected %p but got %p", data, locked_data);
	cr_assert(pthread_mutex_trylock(l.mutex_lock), "Lock is not locked");
	unlock(&l);

	dest_locked(&l, NULL);
}
