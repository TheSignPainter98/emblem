/**
 * @file typesetter.h
 * @brief Provides an interface to call the typesetting loop
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "doc-struct/ast.h"
#include "drivers/drivers.h"

int typeset_doc(Doc* doc, Args* args, OutputDriverInf* driver_inf);
