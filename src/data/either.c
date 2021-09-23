/**
 * @file either.c
 * @brief Provices an implementation of the Either data-type, which can encode values of two types
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "either.h"

#include "pp/not_implemented.h"

void make_either_left(Either* e, void* left_val)
{
	e->type = LEFT;
	e->left = left_val;
}

void make_either_right(Either* e, void* right_val)
{
	e->type	 = RIGHT;
	e->right = right_val;
}

void dest_either(Either* e, Destructor led, Destructor red)
{
	if (e->type == LEFT && led)
		led(e->left);
	else if (red)
		red(e->right);
}

bool succ_either(Either* e) { return e->type != LEFT; }

void fmap_either(Either* eo, Either* ei, Fmap f)
{
	if (ei->type == LEFT)
		make_either_left(eo, ei->left);
	else
	{
		void* o;
		f(&o, ei->right);
		make_either_right(eo, o);
	}
}
