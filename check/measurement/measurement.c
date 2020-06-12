#include "../../src/measurement/measurement.h"

#include <criterion/criterion.h>
#include <math.h>

Test(measurement_checks, area_check)
{
	for (int hei = 0; hei < 10; hei++)
		for (int wid = 0; wid < 10; wid++)
		{
			Dimen dim;
			dim.hei = hei;
			dim.wid = wid;
			cr_assert(area(&dim) == hei * wid, "Area failed correctness assertion\n");
		}
}

Test(measurement_checks, distance_check)
{
	for (int ax = 0; ax < 10; ax++)
		for (int ay = 0; ay < 10; ay++)
			for (int bx = 0; bx < 10; bx++)
				for (int by = 0; by < 10; by++)
				{
					Pos a, b;
					a.x		  = ax;
					a.y		  = ay;
					b.x		  = bx;
					b.y		  = by;
					double dx = ax - bx;
					double dy = ay - by;
					cr_assert(distance(&a, &b) == sqrt((dx * dx) + (dy * dy)), "Distance function didn't return correct euclidean difference magnitude\n");
				}
}
