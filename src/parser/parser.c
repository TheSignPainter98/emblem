#include "parser.h"

#include "data/list.h"
#include "data/locked.h"
#include "doc-struct/location.h"
#include "emblem-parser.h"
#include "logs/logs.h"
#include "pp/unused.h"
#include <stdbool.h>
#include <stdio.h>

void parse_doc(Maybe* mo, List* namesList, Args* args)
{
	Maybe md;
	Locked mtNamesList;
	make_locked(&mtNamesList, namesList);
	log_info("Parsing document '%s'", args->input_file);
	unsigned int nerrs = parse_file(&md, &mtNamesList, args, args->input_file);
	dest_locked(&mtNamesList, NULL);

	if (md.type == NOTHING)
	{
		make_maybe_nothing(mo);
		log_err("Parsing document '%s' failed with %d error%s.", args->input_file, nerrs, nerrs - 1 ? "s" : "");
		return;
	}

	Doc* doc = malloc(sizeof(Doc));
	if (make_doc(doc, md.just, args))
		make_maybe_nothing(mo);
	else
		make_maybe_just(mo, doc);

	dest_maybe(&md, NULL);
}
