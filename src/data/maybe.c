#include "maybe.h"

#include "pp/not_implemented.h"

void make_maybe_nothing(Maybe* m)
{
	m->type	   = NOTHING;
	m->nothing = UNIT;
}

void make_maybe_just(Maybe* m, void* data)
{
	m->type = JUST;
	m->just = data;
}

void dest_maybe(Maybe* m, func_sig(void, ed, (void*)))
{
	if (m && m->type == JUST && ed)
		ed(m->just);
}

bool succ_maybe(Maybe* m) { return m->type != NOTHING; }

void fmap_maybe(Maybe* mo, Maybe* mi, func_sig(void, f, (void**, void*))) {
	if (mi->type == NOTHING)
		make_maybe_nothing(mo);
	else
	{
		void* o;
		f(&o, (void*)mi->just);
		make_maybe_just(mo, o);
	}
}
