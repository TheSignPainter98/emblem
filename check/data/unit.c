#include "data/unit.h"

#include <criterion/criterion.h>

Test(unit, memory_life_cycle)
{
	Unit u;
	make_unit(&u);
	dest_unit(&u);
}

Test(unit, unit_values_equal)
{
	Unit u1;
	Unit u2;
	make_unit(&u1);
	make_unit(&u2);
	cr_assert(u1 == u2, "Differing unit values can be created\n");
	dest_unit(&u1);
	dest_unit(&u2);
}
