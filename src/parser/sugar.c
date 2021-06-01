#include "sugar.h"

#include "pp/unused.h"

void make_sugar(Sugar* sugar, Str* call, size_t src_len)
{
	sugar->call	   = call;
	sugar->src_len = src_len;
}

void dest_sugar(Sugar* sugar) { UNUSED(sugar); }
