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

void dest_either(Either* e __attribute__((unused))) { }

bool succ_either(Either* e) { return e->type != LEFT; }

void fmap_either(Either* eo, Either* ei, func_sig(void, f, (void**, void*)))
{
	if (ei->type == LEFT)
		make_either_left(eo, ei->left);
	else
	{
		void* o;
		f(&o, (void*)ei->right);
		make_either_right(eo, o);
	}
}
