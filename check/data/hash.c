#include "src/data/hash.h"

#include "src/data/str.h"
#include "src/data/tuple.h"
#include "src/pp/assert.h"
#include "src/pp/ignore_warning.h"
#include <criterion/criterion.h>
#include <stdlib.h>

int num_collisions(size_t numHashes, Hash hashes[numHashes]);

#define NUM_HASHES_TO_TEST		  10000
#define NUM_ACCEPTABLE_COLLISIONS ((int)(0.1 * NUM_HASHES_TO_TEST))
#define RAND_SEED				  104

void init_hash_test(void) __attribute__((constructor));
void init_hash_test(void) { srand(RAND_SEED); } // NOLINT

#include <stdio.h>
#define HASH_X_TEST(name, type, ftype, generator)                                                                      \
	Test(hash, type##_collisions)                                                                                      \
	{                                                                                                                  \
		Hash hashes[NUM_HASHES_TO_TEST];                                                                               \
		for (size_t i = 0; i < NUM_HASHES_TO_TEST; i++)                                                                \
		{                                                                                                              \
			int r  = rand();                                                                                           \
			type v = generator;                                                                                        \
			void* w[1];                                                                                                \
			TYPE_PUN_DEREFERENCE(ARRAY_BOUND_MISMATCH(w[0] = *(void**)&v));                                            \
			hashes[i] = hash_##type(*w);                                                                               \
		}                                                                                                              \
		int totCollisions = num_collisions(NUM_HASHES_TO_TEST, hashes);                                                \
		cr_assert(totCollisions <= NUM_ACCEPTABLE_COLLISIONS, "Got %d (>= %d) collisions when hashing %d " #type "s",  \
			totCollisions, NUM_ACCEPTABLE_COLLISIONS, NUM_HASHES_TO_TEST);                                             \
	}

#define HASH_INTEGER_TEST(name, ftype) HASH_X_TEST(name, name, ftype, r)

HASH_INTEGER_TEST(char, "%u")
HASH_INTEGER_TEST(int, "%d")
HASH_INTEGER_TEST(size_t, "%ld")

Test(hash, str_collisions)
{
	Hash hashes[NUM_HASHES_TO_TEST];
	for (size_t i = 0; i < NUM_HASHES_TO_TEST; i++)
	{
		Str s;
		int len = 3 + rand() % 1000;
		char strContent[len + 1];
		for (int i = 0; i < len; i++)
			strContent[i] = (char)rand();
		strContent[len] = '\0';
		make_strv(&s, strContent);
		hashes[i] = hash_str(&s);
		dest_str(&s);
	}
	int totCollisions = num_collisions(NUM_HASHES_TO_TEST, hashes);
	cr_assert(totCollisions <= NUM_ACCEPTABLE_COLLISIONS, "Got %d (>= %d) collisions when hashing %d str s",
		totCollisions, NUM_ACCEPTABLE_COLLISIONS, NUM_HASHES_TO_TEST);
}

int num_collisions(size_t numHashes, Hash hashes[numHashes])
{
	ASSERT(sizeof(Hash) <= sizeof(void*));

	Triple* collisions = calloc(numHashes, sizeof(Triple));

	for (size_t i = 0; i < numHashes; i++)
		for (size_t j = 0; j < numHashes; j++)
		{
			TYPE_PUN_DEREFERENCE(Hash checkingHash = *(Hash*)&collisions[j].p1);
			if (!(bool)collisions[j].p0)
			{
				collisions[j].p0 = (void*)true;
				collisions[j].p1 = *(void**)&hashes[i];
				collisions[j].p2 = (void*)1;
				break;
			}
			else if (checkingHash == hashes[i])
			{
				TYPE_PUN_DEREFERENCE((*(size_t*)&collisions[j].p2)++);
				break;
			}
		}

	size_t maxCollisions = 0;
	for (size_t i = 0; i < numHashes; i++)
		if (maxCollisions < (size_t)collisions[i].p2)
			maxCollisions = (size_t)collisions[i].p2;

	free(collisions);

	return maxCollisions;
}
