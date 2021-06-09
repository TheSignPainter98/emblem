#pragma once

#include "argp.h"
#include "data/list.h"
#include "data/str.h"
#include "ext/ext-env.h"
#include "ext/ext-params.h"
#include "location.h"
#include "style/css-params.h"
#include <libcss/select.h>
#include <stdbool.h>

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
	int flags;
	Str* name;
	Style* style;
	struct DocTreeNodeContent_s* content;
	struct DocTreeNode_s* parent;
	Location* src_loc;
} DocTreeNode;

typedef struct DocTreeNodeContent_s
{
	enum
	{
		WORD,
		CALL,
		CONTENT,
	} type;
	union
	{
		Str* word;
		struct CallIO_s* call;
		List* content;
	};
} DocTreeNodeContent;

extern const char* const node_tree_content_type_names[];
extern const size_t node_tree_content_type_names_len;

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

int make_doc(Doc* doc, DocTreeNode* root, Args* args);
void dest_doc(Doc* doc);

void make_doc_tree_node_word(DocTreeNode* node, Str* word, Location* src_loc);
void make_doc_tree_node_call(DocTreeNode* node, Str* name, CallIO* call, Location* src_loc);
void make_doc_tree_node_content(DocTreeNode* node, Location* src_loc);
void dest_free_doc_tree_node(DocTreeNode* node, bool processing_result);

void dest_doc_tree_node_content(DocTreeNodeContent* content, bool processing_result);

void prepend_doc_tree_node_child(DocTreeNode* parent, List* child_list, DocTreeNode* new_child);

void make_call_io(CallIO* call);
void dest_call_io(CallIO* call, bool processing_result);
void prepend_call_io_arg(CallIO* call, DocTreeNode* arg);
