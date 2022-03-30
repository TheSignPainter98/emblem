#include "src/data/unique-id.h"

#include "data/map.h"
#include <criterion/criterion.h>

#define UNIQUENESS_TESTS 1000

Test(unique_id, is_unique)
{
	Map m;
	make_map(&m, hash_ptr, cmp_ptrs, NULL);
	for (int i = 0; i < UNIQUENESS_TESTS; i++)
	{
		Maybe mo;
		UniqueID id = get_unique_id();
		get_map(&mo, &m, (void*)id);
		cr_assert(mo.type == NOTHING, "UniqueID uniqueness property violated, (got %ld again)", id);
		dest_maybe(&mo, NULL);
	}
	dest_map(&m, NULL);
}
