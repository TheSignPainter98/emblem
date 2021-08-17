#include "style.h"

#include "data/str.h"
#include "debug.h"
#include "logs/logs.h"
#include "lua-pointers.h"
#include "lua.h"
#include "style/css.h"
#include <lauxlib.h>
#include <stdbool.h>

void provide_styler(ExtensionEnv* e)
{
	lua_pushlightuserdata(e->state, e->styler);
	lua_setglobal(e->state, STYLER_LP_LOC);
}
void rescind_styler(ExtensionEnv* e)
{
	lua_pushnil(e->state);
	lua_setglobal(e->state, STYLER_LP_LOC);
}

int ext_import_stylesheet(ExtensionState* s)
{
	dumpstack(s);
	luaL_argcheck(s, true, lua_gettop(s) == 1, "Expected exactly one argument to " EM_IMPORT_STYLESHEET_FUNC_NAME);
	luaL_argcheck(s, true, lua_isstring(s, -1), "Expected string as argument to " EM_IMPORT_STYLESHEET_FUNC_NAME);
	Str sheet_loc;
	char* str = (char*)lua_tostring(s, -1);
	log_debug("Got string at %ld", (size_t)str);
	log_debug("Got string value %s", str);
	make_strv(&sheet_loc, str);

	lua_getglobal(s, STYLER_LP_LOC);
	dumpstack(s);
	if (lua_isnil(s, -1))
		luaL_error(s, "Stylesheets cannot be added after the `start` event has occurred");
	Styler* styler;
	int rc = to_userdata_pointer((void**)&styler, s, -1, STYLER);
	if (rc)
		luaL_error(s, "Invalid internal value");
	lua_pop(s, 1);

	log_debug("Got lua styler at %p", (void*)styler);

	if (append_style_sheet(styler, &sheet_loc))
		luaL_error(s, "Failed to import extension stylesheet '%s'", sheet_loc.str);
	return 0;
}
