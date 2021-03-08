#pragma once

typedef int Unit;
#define UNIT (0)

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
