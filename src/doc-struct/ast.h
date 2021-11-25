/**
 * @file ast.h
 * @brief Exposes functions to handle Emblem document structures
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "argp.h"
#include "config.h"
#include "data/list.h"
#include "data/str.h"
#include "ext/ext-env.h"
#include "location.h"
#include "style/css.h"
#include <libcss/select.h>
#include <stdbool.h>
#include <stdint.h>

// Node names
#define NODE_NAME_PARAGRAPH "p"
#define NODE_NAME_CONTENT	"c"
#define NODE_NAME_WORD		"w"
#define ROOT_NODE_NAME		"body"

#define REQUIRES_RERUN		   (1 << 0)
#define IS_GENERATED_NODE	   (1 << 1)
#define IS_CALL_PARAM		   (1 << 2)
#define CALL_HAS_NO_EXT_FUNC   (1 << 3)
#define CALL_HAS_NO_STYLE	   (1 << 4)
#define PARAGRAPH_CANDIDATE	   (1 << 5)
#define DISQUALIFIED_PARAGRAPH (1 << 6)
#define INCLUDED_FILE_ROOT	   (1 << 7)
#define NO_FURTHER_EVAL		   (1 << 8)
#define STYLE_DIRECTIVE_ONLY   (1 << 9)

#define ACCEPTABLE_EXTENSION_FLAG_MASK                                                                                 \
	(REQUIRES_RERUN | PARAGRAPH_CANDIDATE | DISQUALIFIED_PARAGRAPH | INCLUDED_FILE_ROOT | NO_FURTHER_EVAL              \
		| STYLE_DIRECTIVE_ONLY)

struct DocTreeNodeContent_s;
struct DocTreeNode_s;
struct CallIO_s;
// struct ListContent_s;

typedef struct
{
	struct DocTreeNode_s* root;
	ExtensionEnv* ext;
	Styler* styler;
} Doc;

typedef struct DocTreeNode_s
{
	int_least16_t flags;
	Str* name;
	Str* style_name;
	Style* style;
	StyleData* style_data;
	int last_eval;
	struct DocTreeNodeContent_s* content;
	struct DocTreeNode_s* parent;
	struct DocTreeNode_s* prev_sibling;
	Location* src_loc;
} DocTreeNode;

typedef enum
{
	WORD,
	CALL,
	CONTENT,
} DocTreeNodeContentType;

typedef struct DocTreeNodeContent_s
{
	DocTreeNodeContentType type;
	union
	{
		struct Word_s* word;
		struct CallIO_s* call;
		List* content;
	};
} DocTreeNodeContent;

extern const char* const node_tree_content_type_names[];
extern const size_t node_tree_content_type_names_len;

typedef struct Word_s
{
	Str* raw;
	Str* sanitised;
} Word;

typedef struct CallIO_s
{
	List* args;
	DocTreeNode* result;
} CallIO;

// typedef struct ListContent_s
// {
// enum
// {
// ORDERED,
// UNORDERED,
// } num_type;
// List* items;
// } ListContent;

void make_doc(Doc* doc, DocTreeNode* root, Styler* styler, ExtensionEnv* ext);
void dest_doc(Doc* doc);

void make_doc_tree_node_word(DocTreeNode* node, Str* word, Location* src_loc);
void make_doc_tree_node_call(DocTreeNode* node, Str* name, CallIO* call, Location* src_loc);
void make_doc_tree_node_content(DocTreeNode* node, Location* src_loc);
void dest_free_doc_tree_node(DocTreeNode* node, bool processing_result);

void dest_doc_tree_node_content(DocTreeNodeContent* content, bool processing_result);

void prepend_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child);
void append_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child);

void make_word(Word* word, Str* raw, Location* src_loc);
void dest_word(Word* word);

void make_call_io(CallIO* call);
void dest_call_io(CallIO* call, bool processing_result);
void prepend_call_io_arg(CallIO* call, DocTreeNode* arg);
void append_call_io_arg(CallIO* call, DocTreeNode* arg);

void connect_to_parent(DocTreeNode* restrict child, DocTreeNode* restrict parent);
