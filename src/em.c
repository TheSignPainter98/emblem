#include <stdio.h>

#include "argp.h"

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
	int rc = parse_args(argc, argv);
	if (rc)
		return rc;

	printf("Verbosity is %d\n", Verbose);
	printf("Style = %s\n", Style);
	printf("DefaultTypeface = %s\n", DefaultTypeface);
	printf("DefaultFontSize = %d\n", DefaultFontSize);

	return 0;
}
