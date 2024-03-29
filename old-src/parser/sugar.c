/**
 * @file sugar.c
 * @brief Implements functions to construct syntactic sugar functions
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "sugar.h"

#include "doc-struct/ast.h"
#include "pp/unused.h"
#include <stddef.h>

void make_sugar(Sugar* sugar, Str* call, size_t src_len)
{
	sugar->call	   = call;
	sugar->src_len = src_len;
}

void dest_sugar(Sugar* sugar) { UNUSED(sugar); }

void make_simple_sugar(SimpleSugar* ssugar, Str* call, Str* arg)
{
	ssugar->call = call;
	ssugar->arg  = arg;
}

void make_simple_sugarvc(SimpleSugar* ssugar, char* call, char* arg)
{
	ssugar->call = malloc(sizeof(Str));
	make_strv(ssugar->call, call);
	ssugar->arg = malloc(sizeof(Str));
	make_strc(ssugar->arg, arg);
}

void dest_simple_sugar(SimpleSugar* ssugar) { UNUSED(ssugar); }
