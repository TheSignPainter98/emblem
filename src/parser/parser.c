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
	parse_file(&md, &mtNamesList, args, args->input_file);
	dest_locked(&mtNamesList, NULL);

	if (md.type == NOTHING)
	{
		make_maybe_nothing(mo);
		return;
	}

	Doc* doc = malloc(sizeof(Doc));
	if (make_doc(doc, md.just, args))
		make_maybe_nothing(mo);
	else
		make_maybe_just(mo, doc);

	dest_maybe(&md, NULL);
}
