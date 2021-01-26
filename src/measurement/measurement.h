#pragma once

typedef struct
{
	double wid;
	double hei;
} Dimen;

double area(Dimen* dim) __attribute__((pure));

/**
 * @brief A position in 2D space
 */
typedef struct
{
	/**
	 * @brief Coordinate along the first axis
	 */
	double x;
	/**
	 * @brief Coorinate along the second axis
	 */
	double y;
} Pos;

/**
 * @brief Compute the distance between two cartesian points
 *
 * @param a Some point
 * @param b Some other point
 *
 * @return The Euclidean distance between a and b
 */
double distance(Pos* a, Pos* b) __attribute__((pure));
