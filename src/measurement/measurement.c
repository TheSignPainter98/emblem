#include "measurement.h"

#include <math.h>

double distance(Pos* a, Pos* b)
{
	double dx = a->x - b->x;
	double dy = a->y - b->y;
	return sqrt(dx * dx + dy * dy);
}
