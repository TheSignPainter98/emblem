#include "typesetter.h"

#include "ext/ext-params.h"
#include "ext/lua-events.h"
#include "ext/lua.h"
#include "ext/style.h"
#include "logs/logs.h"
#include "style/css.h"

int typeset_doc(Doc* doc, Args* args, OutputDriverInf* driver_inf)
{
	int rc;
	ExtParams ext_params;
	init_ext_params(&ext_params, args);

	if (make_doc_ext_state(doc, &ext_params))
		return 1;
	if (do_lua_start_event(doc->ext->state))
	{
		dest_doc_ext_state(doc);
		return 1;
	}

	rescind_styler(doc->ext);

	if (prepare_styler(doc->styler))
	{
		return 1;
	}

	do
	{
		inc_iter_num(doc->ext);
		log_info("Executing iteration %d", doc->ext->iter_num);
		doc->ext->require_extra_run = false;
		if (do_lua_iter_start_event(doc->ext->state))
		{
			dest_doc_ext_state(doc);
			return 1;
		}
		rc = exec_lua_pass(doc);
		if (rc)
			return rc;

		if (driver_inf->supports_typesetting)
			log_debug("Executing typesetting pass %d", doc->ext->iter_num);

		if (do_lua_iter_end_event(doc->ext->state))
		{
			dest_doc_ext_state(doc);
			return 1;
		}
	} while (doc->ext->require_extra_run & (doc->ext->iter_num < args->max_iters));

	if (doc->ext->iter_num == args->max_iters)
		log_warn("Max number of typesetting iterations reached (%d)", args->max_iters);

	if (do_lua_end_event(doc->ext->state))
	{
		dest_doc_ext_state(doc);
		return 1;
	}
	dest_doc_ext_state(doc);
	return 0;
}
