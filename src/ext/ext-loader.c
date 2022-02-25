/**
 * @file ext-loader.c
 * @brief Implements the extension-loader
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "ext-loader.h"

#include "data/list.h"
#include "data/map.h"
#include "data/maybe.h"
#include "data/str.h"
#include "logs/logs.h"
#include <lauxlib.h>
#include <string.h>

#define EXTENSION_ARG_TABLE "em_ext_args"

static int load_extension(ExtensionState* s, Str* ext_name);
static int load_extension_code(ExtensionState* s, Str* ext_name);

int load_extensions(ExtensionState* s, ExtParams* ext_params)
{
	log_info("Loading extensions...");

	ListIter li;
	make_list_iter(&li, ext_params->exts);
	Str* extName;
	int rc;
	while (iter_list((void**)&extName, &li))
		if ((rc = load_extension(s, extName)))
			return rc;

	return 0;
}

static int load_extension(ExtensionState* s, Str* ext_name)
{
	log_info("Loading extension '%s'", ext_name->str);
	return load_extension_code(s, ext_name);
}

static int load_extension_code(ExtensionState* s, Str* ext_name)
{
	const char* const default_extension =  ".lua";
	char ext_name_path[ext_name->len + strlen(default_extension)];
	memcpy(ext_name_path, ext_name->str, ext_name->len);
	if (!strrchr(ext_name->str, '.'))
		strcpy(ext_name_path + ext_name->len, default_extension); // NOLINT
	else
		ext_name_path[ext_name->len] = '\0';

	int lrc = luaL_loadfile(s, ext_name_path);
	if (lrc)
	{
		log_err("Failed to load file '%s' (%d): %s", ext_name_path, lrc, lua_tostring(s, -1));
		return 1;
	}
	lrc = lua_pcall(s, 0, 0, 0);
	if (lrc)
	{
		log_err("Failed to load file '%s' (%d): %s", ext_name_path, lrc, lua_tostring(s, -1));
		return 1;
	}
	return 0;
}
