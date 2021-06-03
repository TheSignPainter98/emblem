#include "drivers.h"

#include "data/cmp.h"
#include "html.h"
#include "logs/logs.h"
#include "pp/lambda.h"
#include "pp/unused.h"
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>

#define DRIVER_INFO_FUNC_NAME	"driver_info"
#define DRIVER_RUNNER_FUNC_NAME "run_driver"
#define DRIVER_LIB_NAME_FMT		"libem-%s.so"

static int init_internal_driver(OutputDriver* driver, InternalDriver* idriver, Args* args);
static int init_external_driver(OutputDriver* driver, Args* args);
static void dest_output_driver_inf(OutputDriverInf* inf);
static void strip_ext(char* fname);

typedef int (*InternalDriverGetter)(InternalDriver*);
typedef struct
{
	const char* const name;
	InternalDriverGetter get_driver;
} InternalDriverDecl;

InternalDriverDecl internal_drivers[] = {
	{ "html", make_html_driver },
};

int get_output_driver(OutputDriver* driver, Args* args)
{
	log_info("Loading driver '%s'", args->driver);

	bool is_internal_driver = false;
	InternalDriver idriver;
	for (size_t i = 0; i < sizeof(internal_drivers) / sizeof(*internal_drivers); i++)
		if (streq(internal_drivers[i].name, args->driver))
		{
			is_internal_driver = true;
			int rc			   = internal_drivers[i].get_driver(&idriver);
			if (rc)
				return rc;
			break;
		}

	if (is_internal_driver)
		return init_internal_driver(driver, &idriver, args);
	return init_external_driver(driver, args);
}

static int init_internal_driver(OutputDriver* driver, InternalDriver* idriver, Args* args)
{
	driver->type = INTERNAL;

	driver->driver_lib_name = malloc(sizeof(Str));
	make_strv(driver->driver_lib_name, "<internal>");
	driver->driver_name = malloc(sizeof(Str));
	make_strv(driver->driver_name, args->driver);
	driver->lib_handle = NULL;

	driver->inf = idriver->inf;
	driver->run = idriver->run;

	return 0;
}

static int init_external_driver(OutputDriver* driver, Args* args)
{
	driver->type = EXTERNAL;

	const size_t lib_name_len = 1 + snprintf(NULL, 0, DRIVER_LIB_NAME_FMT, args->driver);
	char* lib_name			  = malloc(lib_name_len);
	snprintf(lib_name, lib_name_len, DRIVER_LIB_NAME_FMT, args->driver);

	driver->driver_name = malloc(sizeof(Str));
	make_strv(driver->driver_name, args->driver);
	driver->driver_lib_name = malloc(sizeof(Str));
	make_strv(driver->driver_lib_name, lib_name);

	log_debug("Searching for driver library '%s'", lib_name);
	driver->lib_handle = dlopen(lib_name, RTLD_LAZY);
	char* err		   = dlerror();
	if (!driver->lib_handle)
	{
		log_err("Could not open library '%s': %s", args->driver, err);
		return 1;
	}

	DriverInfGetter inf_getter;
	*(void**)(&inf_getter) = dlsym(driver->lib_handle, DRIVER_INFO_FUNC_NAME);
	if ((err = dlerror()))
	{
		log_err("Could not obtain symbol '%s' from %s: %s", DRIVER_INFO_FUNC_NAME, driver->driver_lib_name->str, err);
		return 1;
	}

	*(void**)(&driver->run) = dlsym(driver->lib_handle, DRIVER_RUNNER_FUNC_NAME);
	if ((err = dlerror()))
	{
		log_err("Could not obtain symbol '%s' from %s: %s", DRIVER_RUNNER_FUNC_NAME, driver->driver_lib_name->str, err);
		return 1;
	}

	// Get the output info
	driver->inf = malloc(sizeof(OutputDriverInf));
	return inf_getter(driver->inf);
}

void dest_output_driver(OutputDriver* driver)
{
	dest_str(driver->driver_name);
	free(driver->driver_name);
	dest_str(driver->driver_lib_name);
	free(driver->driver_lib_name);
	if (driver->lib_handle)
		dlclose(driver->lib_handle);
	dest_output_driver_inf(driver->inf);
	free(driver->inf);
}

static void dest_output_driver_inf(OutputDriverInf* inf) { UNUSED(inf); }

void make_driver_params(DriverParams* params, Args* args)
{
	params->output_stem = malloc(sizeof(Str));
	if (!streq(args->output_stem, ""))
		make_strv(params->output_stem, args->output_stem);
	else if (streq(args->input_file, "-"))
		make_strv(params->output_stem, "emdoc");
	else
	{
		// Remove extension from the input and use that as the default
		char* inoext = strdup(args->input_file);
		strip_ext(inoext);
		make_strr(params->output_stem, inoext);
	}
}

static void strip_ext(char* fname)
{
	char* end = fname + strlen(fname);
	while (end > fname && *end != '.' && *end != '/' && *end != '\\')
		end--;
	if (end > fname && *end == '.' && *(end - 1) != '/' && *(end - 1) != '\\')
		*end = '\0';
}

void dest_driver_params(DriverParams* params)
{
	dest_str(params->output_stem);
	free(params->output_stem);
}
