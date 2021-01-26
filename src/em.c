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
	log_err("Hello, world %s", "how are you?");
	log_warn("Hello, world %s", "how are you?");
	log_info("Hello, world %s", "how are you?");
	log_succ("Hello, world %s", "how are you?");

	log_succ("Got default font %s @ %dpt", args.default_typeface, args.default_font_size);

	return 0;
}
