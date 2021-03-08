#pragma once

/**
 * @brief Unit type, has only one value value and hence conveys no information
 */
typedef int Unit;

/**
 * @brief Value of the unit type
 *
 * @return The unique valid value of the unit type
 */
#define UNIT 0

/**
 * @brief Destroy a unit
 *
 * @param unitp Pointer to a unit to destroy
 */
void dest_unit(Unit* unitp);

/**
 * @brief Make a unit
 *
 * @param unitp Pointer to the unit to make
 */
void make_unit(Unit* unitp);
