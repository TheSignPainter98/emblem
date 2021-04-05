#include "parser.h"

#include "em-emblem-parser.h"
#include "logs/logs.h"
#include <stdio.h>

void parse_doc(Maybe* mo, Args* args)
{
	parse_file(mo, args, args->input_file);
}
