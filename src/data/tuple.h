#pragma once

#include "unit.h"

typedef struct
{
	Unit unit_;
} EmptyTuple;

typedef struct
{
	void* p0;
} Singleton;

typedef struct
{
	void* p0;
	void* p1;
} Pair;

typedef struct
{
	void* p0;
	void* p1;
	void* p2;
} Triple;

typedef struct
{
	void* p0;
	void* p1;
	void* p2;
	void* p3;
} Quadruple;

typedef struct
{
	void* p0;
	void* p1;
	void* p2;
	void* p3;
	void* p4;
} Quintuple;

typedef struct
{
	void* p0;
	void* p1;
	void* p2;
	void* p3;
	void* p4;
	void* p5;
} Sextuple;

typedef struct
{
	void* p0;
	void* p1;
	void* p2;
	void* p3;
	void* p4;
	void* p5;
	void* p6;
} Septuple;

typedef struct
{
	void* p0;
	void* p1;
	void* p2;
	void* p3;
	void* p4;
	void* p5;
	void* p6;
	void* p7;
} Octuple;
