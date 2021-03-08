#pragma once

#include <stddef.h>

typedef unsigned int Hash;
typedef Hash (*Hasher)(void*);

#define HASH_SIG(name) Hash hash_##name(void* v)

HASH_SIG(char);
HASH_SIG(int);
HASH_SIG(size_t);
HASH_SIG(ptr);
HASH_SIG(str);
