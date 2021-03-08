#pragma once

typedef enum
{
	LT = -1,
	EQ = 0,
	GT = 1
} Cmp;
typedef Cmp (*Comparator)(void*, void*);

#define CMP_SIG(name) Cmp cmp_## name ##s(void* v1, void* v2)

CMP_SIG(char);
CMP_SIG(double);
CMP_SIG(float);
CMP_SIG(int);
CMP_SIG(size_t);
CMP_SIG(ptr);
CMP_SIG(str);
