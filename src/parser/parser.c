#include "parser.h"

#include "data/list.h"
#include "data/locked.h"
#include "doc-struct/location.h"
#include "emblem-parser.h"
#include "logs/logs.h"
#include "pp/unused.h"
#include <stdbool.h>
#include <stdio.h>

void parse_doc(Maybe* mo, Locked* mtNamesList, Args* args)
{
	log_info("Parsing document '%s'", args->input_file);
	unsigned int nerrs = parse_file(mo, mtNamesList, args, args->input_file);

	if (mo->type == NOTHING)
	{
		make_maybe_nothing(mo);
		log_err("Parsing document '%s' failed with %d error%s.", args->input_file, nerrs, nerrs - 1 ? "s" : "");
	}
}
