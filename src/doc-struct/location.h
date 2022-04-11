/**
 * @file location.h
 * @brief Exposes functions to handle the Location structure for keeping track of places in the document source
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "data/str.h"
#include "data/unique-id.h"
#include "ext/ext-env.h"
#include "shared-destruction.h"
#include <stdbool.h>
#include <stddef.h>

#define LOC_ID(node) ((lua_Integer)node)

typedef struct
{
	UniqueID id;
	size_t first_line;
	size_t first_column;
	size_t last_line;
	size_t last_column;
	Str* src_file;
	bool has_ep;
	bool has_node_ref;
	bool owns_src_file;
} Location;

void make_location(Location* loc, size_t first_line, size_t first_column, size_t last_line, size_t last_column,
	Str* src_file, bool owns_src_file);
void dest_free_location(Location* loc, SharedDestructionMode shared_mode);

Location* dup_loc(Location* todup, bool force_dup_src_file);
void register_ext_location(ExtensionState* s);
Location* node_loc_ref(Location* loc);
void push_location(ExtensionState* s, Location* loc);
