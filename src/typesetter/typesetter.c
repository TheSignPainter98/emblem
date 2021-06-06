#include "typesetter.h"

#include "ext/ext-env.h"
#include "ext/ext-params.h"
#include "ext/lua-events.h"
#include "ext/lua.h"
/* #include "ext/style.h" */
#include "logs/logs.h"
#include "style/css.h"

int typeset_doc(Doc* doc, Args* args, OutputDriverInf* driver_inf)
{
	int rc;

	if (do_lua_start_event(doc->ext->state))
		return 1;

	finalise_env_for_typesetting(doc->ext);

	if (prepare_styler(doc->styler))
		return 1;

	do
	{
		inc_iter_num(doc);
		log_info("Executing iteration %d", doc->ext->iter_num);
		doc->ext->require_extra_run = false;
		if (do_lua_iter_start_event(doc->ext->state))
			return 1;
		rc = exec_lua_pass(doc);
		if (rc)
			return rc;

		if (driver_inf->support & TS_PLACEMENT)
			log_debug("Executing typesetting pass %d", doc->ext->iter_num);

		if (do_lua_iter_end_event(doc->ext->state))
			return 1;
	} while (doc->ext->require_extra_run & (doc->ext->iter_num < args->max_iters));

	if (doc->ext->iter_num == args->max_iters)
		if (log_warn("Max number of typesetting iterations reached (%d)", args->max_iters))
			return 1;

	if (do_lua_end_event(doc->ext->state))
		return 1;
	return 0;
}
