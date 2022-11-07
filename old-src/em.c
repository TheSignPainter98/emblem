/**
 * @file em.c
 * @brief Insertion point for the 'em' binary, invokes all functionality used for typeestting documents
 * @author Edward Jones
 * @date 2021-09-17
 */
#include <stdio.h>

#include "argp.h"
#include "data/destructor.h"
#include "data/list.h"
#include "data/locked.h"
#include "data/maybe.h"
#include "doc-struct/ast.h"
#include "drivers/drivers.h"
#include "ext/ext-env.h"
#include "ext/ext-params.h"
#include "ext/setting-io.h"
#include "logs/logs.h"
#include "parser/parser.h"
#include "style/css.h"
#include "style/styler-driver-interface.h"
#include "typesetter/typesetter.h"

/**
 * @brief Entry point
 *
 * @param argc Number of command-line arguments
 * @param argv Command-line argument array
 *
 * @return Program exit code
 */
int main(int argc, char** argv)
{
	int rc = 0;

	// Parse arguments, exit if necessary
	Args args;
	rc = parse_args(&args, argc, argv);
	if (rc)
	{
		if (rc < 0)
			rc = 0;
		goto clean_args;
	}
	init_logs(&args);

	Styler styler;
	make_styler(&styler, &args);
	List namesList;
	make_list(&namesList);
	Locked mtNamesList;
	make_locked(&mtNamesList, &namesList);
	ExtParams ext_params;
	make_ext_params(&ext_params, &args, &styler, &mtNamesList);
	ExtensionEnv ext;
	if ((rc = make_ext_env(&ext, &ext_params)))
		goto clean_args;

	const char* input = args.input_file;
	if (streq(input, "-"))
	{
		const char* conf_main = get_setting(&ext, "main");
		if (conf_main)
			input = conf_main;
	}

	// Get the output driver
	OutputDriver driver;
	if ((rc = get_output_driver(&driver, &args, &ext)))
		goto clean_ext;

	pass_output_driver_data_to_styler(&styler, &driver);

	// Parse the document
	Maybe maybe_ast_root;
	parse_doc(&maybe_ast_root, &mtNamesList, &args, input);
	if ((rc = maybe_ast_root.type == NOTHING))
		goto clean_output_driver;

	DocTreeNode* root = maybe_ast_root.just;
	Doc doc;
	make_doc(&doc, root, &styler, &ext);
	rc = typeset_doc(&doc, &args, driver.support);
	if (rc)
		goto cleanup;

	log_info("Executing output driver");
	if ((rc = run_output_driver(&driver, &doc, &ext)))
		goto cleanup;

cleanup:
	log_debug("Cleaning up execution");
	dest_doc(&doc);
clean_output_driver:
	dest_output_driver(&driver);
clean_ext:
	if (input != args.input_file)
		release_setting(&ext);
	dest_ext_env(&ext);
	dest_ext_params(&ext_params);
	dest_locked(&mtNamesList, NULL);
	dest_list(&namesList, (Destructor)dest_free_str);
	dest_styler(&styler);
	fini_logs();
clean_args:
	dest_args(&args);

	return rc;
}
