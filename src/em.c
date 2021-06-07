#include <stdio.h>

#include "argp.h"
#include "data/destructor.h"
#include "data/list.h"
#include "data/maybe.h"
#include "logs/logs.h"
#include "parser/parser.h"
#include "typesetter/typesetter.h"

static void doc_destroyer(Doc* doc);
static void str_destroyer(Str* str);

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
	int rc = 0;

	// Parse arguments, exit if necessary
	Args args;
	rc = parse_args(&args, argc, argv);
	if (rc)
		return rc;
	init_logs(&args);

	// Get the output driver
	OutputDriver driver;
	rc = get_output_driver(&driver, &args);
	if (rc)
		return rc;

	// Parse the document
	List namesList;
	make_list(&namesList);
	Maybe mdoc;
	parse_doc(&mdoc, &namesList, &args);
	rc = mdoc.type == NOTHING;
	if (rc)
		return rc;

	Doc* doc = mdoc.just;
	rc		 = typeset_doc(doc, &args, driver.inf);
	if (rc)
		return rc;

	DriverParams driver_params;
	make_driver_params(&driver_params, &args);
	rc = driver.run(doc, &driver_params);
	if (rc)
		return rc;

	log_debug("Cleaning up execution");
	dest_driver_params(&driver_params);
	dest_maybe(&mdoc, (Destructor)doc_destroyer);
	dest_list(&namesList, (Destructor)str_destroyer);
	dest_output_driver(&driver);
	dest_args(&args);
	return rc;
}

static void doc_destroyer(Doc* doc)
{
	dest_doc(doc);
	free(doc);
}

static void str_destroyer(Str* str)
{
	dest_str(str);
	free(str);
}
