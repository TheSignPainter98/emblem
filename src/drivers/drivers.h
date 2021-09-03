#pragma once

#include "argp.h"
#include "driver-params.h"
#include "ext/ext-env.h"

int get_output_driver(OutputDriver* driver, Args* args, ExtensionEnv* ext);
void dest_output_driver(OutputDriver* driver);
int run_output_driver(OutputDriver* driver, Doc* doc, ExtensionEnv* ext);
