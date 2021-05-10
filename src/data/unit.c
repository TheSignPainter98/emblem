#include "unit.h"

#include "pp/unused.h"

void make_unit(Unit* unitp) { *unitp = UNIT; }

void dest_unit(Unit* unitp) { UNUSED(unitp); }
