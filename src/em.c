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
#include "logs/logs.h"
#include "parser/parser.h"
#include "style/css.h"
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
		return rc;
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
		return rc;

	// Get the output driver
	OutputDriver driver;
	rc = get_output_driver(&driver, &args, &ext);
	if (rc)
		return rc;

	// Parse the document
	Maybe maybe_ast_root;
	parse_doc(&maybe_ast_root, &mtNamesList, &args);
	rc = maybe_ast_root.type == NOTHING;
	if (rc)
		return rc;

	DocTreeNode* root = maybe_ast_root.just;
	Doc doc;
	make_doc(&doc, root, &styler, &ext);
	rc = typeset_doc(&doc, &args, driver.support);
	if (rc)
		return rc;

	log_info("Executing output driver");
	rc = run_output_driver(&driver, &doc, &ext);
	if (rc)
		return rc;

	log_debug("Cleaning up execution");
	dest_doc(&doc);
	dest_ext_env(&ext);
	dest_output_driver(&driver);
	dest_locked(&mtNamesList, NULL);
	dest_list(&namesList, (Destructor)dest_free_str);
	dest_ext_params(&ext_params);
	dest_styler(&styler);
	dest_args(&args);
	return rc;
}
