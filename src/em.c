#include <stdio.h>

#include "argp.h"
#include "data/maybe.h"
#include "logs/logs.h"
#include "parser/parser.h"

/**
 * @brief Entry point
 *
 * @param argc Number of command-line arguments
 * @param argv Command-line argument array
 *
 * @return
 */
int main(int argc, char** argv)
{
	// Parse arguments, exit if necessary
	Args args;
	int rc = parse_args(&args, argc, argv);
	if (rc)
		return rc;
	init_logs(&args);

	Maybe edoc;
	parse_doc(&edoc, &args);
	dest_maybe(&edoc, NULL);

	dest_args(&args);
	return 0;
}
