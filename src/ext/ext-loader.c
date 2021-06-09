#include "ext-loader.h"

#include "data/list.h"
#include "data/map.h"
#include "data/maybe.h"
#include "data/str.h"
#include "logs/logs.h"
#include <lauxlib.h>
#include <string.h>

#define EXTENSION_ARG_TABLE "em_ext_args"

static int read_arg_defs_into_map(Map* arg_list_map, List* ext_args);
static int parse_arg_def(Str* argDef, Str* ext_name, Str* param_name, Str* arg_val);
static char* nstrchr(char* s, int c);
static int load_extension(ExtensionState* s, Str* ext_name, Maybe* params);
static int load_extension_arguments(ExtensionState* s, Maybe* params);
static int load_extension_code(ExtensionState* s, Str* ext_name);

int load_extensions(ExtensionState* s, ExtParams* ext_params)
{
	log_info("Loading extensions...");

	Map argLists;
	make_map(&argLists, hash_str, cmp_strs, (Destructor)dest_free_str);
	int rc = read_arg_defs_into_map(&argLists, ext_params->ext_args);
	if (rc)
		return 1;

	ListIter li;
	make_list_iter(&li, ext_params->exts);
	Str* extName;
	while (iter_list((void**)&extName, &li))
	{
		Maybe argList;
		get_map(&argList, &argLists, extName);
		int rc = load_extension(s, extName, &argList);
		if (rc)
			return rc;
		dest_maybe(&argList, NULL);
	}

	NON_ISO(Destructor vd = (Destructor)ilambda(void, (List * arg_list),
				{
					NON_ISO(Destructor pd = (Destructor)ilambda(void, (Pair * pa),
								{
									free(pa->p0);
									free(pa->p1);
									free(pa);
								}));
					dest_list(arg_list, pd);
					free(arg_list);
				}));
	dest_map(&argLists, vd);

	return 0;
}

static int read_arg_defs_into_map(Map* arg_list_map, List* ext_args)
{
	int rc = 0;

	ListIter li;
	make_list_iter(&li, ext_args);
	Str* argDef;
	while (iter_list((void**)&argDef, &li))
	{
		Str* ext_name	= malloc(sizeof(Str));
		Str* param_name = malloc(sizeof(Str));
		Str* arg		= malloc(sizeof(Str));
		int rc2			= parse_arg_def(argDef, ext_name, param_name, arg);
		rc |= rc2;
		if (rc2)
			continue;

		List* arg_list;
		Maybe m;
		get_map(&m, arg_list_map, ext_name);
		switch (m.type)
		{
			case JUST:
				arg_list = m.just;
				dest_free_str(ext_name);
				break;
			case NOTHING:
				arg_list = malloc(sizeof(List));
				make_list(arg_list);
				Maybe mr;
				push_map(&mr, arg_list_map, ext_name, arg_list);
				dest_maybe(&mr, NULL);
				break;
			default:
				log_err("Getting arg list from map returned an object of unknown maybe-type: %d", m.type);
				exit(1);
		}

		Pair* pa = malloc(sizeof(Pair));
		pa->p0	 = param_name;
		pa->p1	 = arg;
		append_list(arg_list, pa);

		dest_maybe(&m, NULL);
	}

	return rc;
}

static int parse_arg_def(Str* argDef, Str* ext_name, Str* param_name, Str* arg_val)
{
	char* raw_ext_name	 = argDef->str;
	char* raw_param_name = nstrchr(raw_ext_name, '.');
	char* raw_arg_val	 = nstrchr(raw_param_name, '=');
	if (!raw_param_name)
	{
		log_err("No parameter specified in '%s': missing '.' character", argDef->str);
		return 1;
	}
	if (!raw_arg_val)
	{
		log_err("No argument specified in '%s': missing '=' character after '.'", argDef->str);
		return 1;
	}

	*raw_param_name++ = '\0';
	*raw_arg_val++	  = '\0';

	make_strv(ext_name, raw_ext_name);
	make_strv(param_name, raw_param_name);
	make_strv(arg_val, raw_arg_val);
	return 0;
}

static char* nstrchr(char* s, int c)
{
	if (s)
		return strchr(s, c);
	return s;
}

static int load_extension(ExtensionState* s, Str* ext_name, Maybe* params)
{
	log_info("Loading extension '%s'", ext_name->str);
	int rc = load_extension_arguments(s, params);
	if (rc)
		return rc;
	return load_extension_code(s, ext_name);
}

static int load_extension_arguments(ExtensionState* s, Maybe* params)
{
	List* arg_list = params->just;

	lua_newtable(s);

	if (params->type == JUST)
	{
		ListIter li;
		make_list_iter(&li, arg_list);
		Pair* pa;
		while (iter_list((void**)&pa, &li))
		{
			Str* param = pa->p0;
			Str* arg   = pa->p1;
			lua_pushstring(s, arg->str);
			lua_setfield(s, -2, param->str);
		}
	}

	lua_setglobal(s, EXTENSION_ARG_TABLE);

	return 0;
}

static int load_extension_code(ExtensionState* s, Str* ext_name)
{
	// TODO: Make the .lua optional
	char ext_name_path[ext_name->len + 3];
	memcpy(ext_name_path, ext_name->str, ext_name->len);
	strncpy(ext_name_path + ext_name->len, ".lua", 5);

	int lrc = luaL_loadfile(s, ext_name_path);
	if (lrc)
	{
		const char* err = lua_tostring(s, 0);
		log_err("Failed to load file '%s' (%d): %s", ext_name_path, lrc, err);
		return 1;
	}
	lrc = lua_pcall(s, 0, 0, 0);
	if (lrc)
	{
		const char* err = lua_tostring(s, 0);
		log_err("Failed to load file '%s' (%d): %s", ext_name_path, lrc, err);
		return 1;
	}
	return 0;
}
