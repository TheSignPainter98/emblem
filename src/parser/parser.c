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
	Locked mtNamesList;
	make_locked(&mtNamesList, namesList);
	log_info("Parsing document '%s'", args->input_file);
	parse_file(mo, &mtNamesList, args, args->input_file);
	dest_locked(&mtNamesList, NULL);
}
