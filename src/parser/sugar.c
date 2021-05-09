#include "sugar.h"

void make_sugar(Sugar* sugar, Str* call, size_t src_len)
{
	sugar->call	   = call;
	sugar->src_len = src_len;
}

void dest_sugar(Sugar* sugar)
{
	dest_str(sugar->call);
	free(sugar->call);
}
