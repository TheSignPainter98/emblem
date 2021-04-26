#pragma once

#include "argp.h"
#include "driver-params.h"

typedef int (*DriverInfGetter)(OutputDriverInf* inf);

int get_output_driver(OutputDriver* driver, Args* args);
void dest_output_driver(OutputDriver* driver);

void make_driver_params(DriverParams* params, Args* args);
void dest_driver_params(DriverParams* params);
