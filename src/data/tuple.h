#pragma once

#include "unit.h"

/**
 * @brief An empty tuple
 */
typedef struct
{
	/**
	 * @brief The sole element of the empty tuple. This is a placeholder.
	 */
	Unit unit_;
} EmptyTuple;

/**
 * @brief A tuple with one element
 */
typedef struct
{
	/**
	 * @brief The sole element of the singleton tuple
	 */
	void* p0;
} Singleton;

/**
 * @brief A pair of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
} Pair;

/**
 * @brief A triple of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
	/**
	 * @brief Third value
	 */
	void* p2;
} Triple;

/**
 * @brief A 4-tuple of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
	/**
	 * @brief Third value
	 */
	void* p2;
	/**
	 * @brief Fourth value
	 */
	void* p3;
} Quadruple;

/**
 * @brief A 5-tuple of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
	/**
	 * @brief Third value
	 */
	void* p2;
	/**
	 * @brief Fourth value
	 */
	void* p3;
	/**
	 * @brief Fifth value
	 */
	void* p4;
} Quintuple;

/**
 * @brief A 6-tuple of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
	/**
	 * @brief Third value
	 */
	void* p2;
	/**
	 * @brief Fourth value
	 */
	void* p3;
	/**
	 * @brief Fifth value
	 */
	void* p4;
	/**
	 * @brief Sixth value
	 */
	void* p5;
} Sextuple;

/**
 * @brief A 7-tuple of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
	/**
	 * @brief Third value
	 */
	void* p2;
	/**
	 * @brief Fourth value
	 */
	void* p3;
	/**
	 * @brief Fifth value
	 */
	void* p4;
	/**
	 * @brief Sixth value
	 */
	void* p5;
	/**
	 * @brief Seventh value
	 */
	void* p6;
} Septuple;

/**
 * @brief An 8-tuple of values
 */
typedef struct
{
	/**
	 * @brief First value
	 */
	void* p0;
	/**
	 * @brief Second value
	 */
	void* p1;
	/**
	 * @brief Third value
	 */
	void* p2;
	/**
	 * @brief Fourth value
	 */
	void* p3;
	/**
	 * @brief Fifth value
	 */
	void* p4;
	/**
	 * @brief Sixth value
	 */
	void* p5;
	/**
	 * @brief Seventh value
	 */
	void* p6;
	/**
	 * @brief Eighth value
	 */
	void* p7;
} Octuple;
