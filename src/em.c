#include <stdio.h>

#include "argp.h"
#include "logs/logs.h"

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
	dest_args(&args);
	return 0;
}
