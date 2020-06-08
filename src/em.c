#include <stdio.h>

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
	printf("Got %d; called with %s", argc, *argv);
	fprintf(stderr, "Got %d; called with %s", argc, *argv);
	log_warn("Got %d; called with %s", argc, *argv);
	log_err("Got %d; called with %s", argc, *argv);

	return 0;
}
