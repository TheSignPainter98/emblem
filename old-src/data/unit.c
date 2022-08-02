/**
 * @file unit.c
 * @brief Implements the Unit data type
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "unit.h"

#include "pp/unused.h"

void make_unit(Unit* unitp) { *unitp = UNIT; }

void dest_unit(Unit* unitp) { UNUSED(unitp); }
