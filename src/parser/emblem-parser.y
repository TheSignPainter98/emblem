%code requires {
#include "argp.h"
#include "parser.h"
#include "data/locked.h"
#include "data/str.h"
#include "sugar.h"
unsigned int parse_file(Maybe* eo, Locked* namesList, Args* args, char* fname);

typedef struct
{
	char* fname;
	int line_num;
	int line_col;
	DocTreeNode* included_root;
} PreProcessorData;

typedef struct
{
	int comment_lvl;
	int indent_lvl;
	int indent_lvl_target;
	bool post_dent_tok_required;
	int post_dent_tok;
	int tab_size;
	bool opening_emph;
	int* nerrs;
	FILE* ifp;
	Str* ifn;
	Locked* mtNamesList;
	Args* args;
	PreProcessorData preproc;
} LexerData;

typedef struct
{
	Args* args;
	DocTreeNode* root;
	Str** ifn;
	int* nerrs;
	void** scanner;
} ParserData;
}

%{
#include "pp/unused.h"

#include "data/str.h"
#include "data/either.h"
#include "emblem-parser.h"
#include "logs/logs.h"
#include "emblem-lexer.h"
#include <stdbool.h>
#include <string.h>

#define YYLEX_PARAM_ data->scanner
#define NON_MATCHING_PAIR_ERROR_MSG "Unpaired closing emphasis length, expected %ld characters but got %ld"
#define ENSURE_MATCHING_PAIR(opening_sugar, close_len, close_tok_loc, params)\
	const size_t open_len = opening_sugar.src_len;\
	if (open_len != close_len)\
	{\
		size_t msg_len = 1 + snprintf(NULL, 0, NON_MATCHING_PAIR_ERROR_MSG, open_len, close_len);\
		char msg[msg_len];\
		sprintf(msg, NON_MATCHING_PAIR_ERROR_MSG, open_len, close_len);\
		yyerror(&close_tok_loc, params, msg);\
		YYERROR;\
	}
#define DEFAULT_CONTENT_EXTENSION ".em"
%}

%define			parse.trace true
%define			parse.error verbose
%define			api.pure    full
%define			api.prefix  {em_}
%define			lr.type		ielr
%parse-param				{ ParserData* data }
%lex-param					{ yyscan_t YYLEX_PARAM_ }
%locations
%expect 0

%union {
	Unit doc;
	DocTreeNode* node;
	CallIO* args;
	Str* str;
	Sugar sugar;
	SimpleSugar simple_sugar;
	size_t len;
}

%nterm <args>			short_args
%nterm <args>			line_remainder_args
%nterm <args>			multi_line_args
%nterm <args>			trailing_args
%nterm <args> 			bullet_list_lines_ne
%nterm <args> 			bullet_list_lines
%nterm <args> 			sequenced_list_lines_ne
%nterm <args> 			sequenced_list_lines
%nterm <doc>			doc
%nterm <node> 			bulleted_list
%nterm <node>			doc_content
%nterm <node>			file_content
%nterm <node>			file_contents
%nterm <node>			line
%nterm <node>			line_content
%nterm <node>			line_content_ne
%nterm <node>			line_element
%nterm <node>			lines
%nterm <node>			par
%nterm <node> 			sequenced_list
%token					T_DEDENT			"dedent"
%token					T_GROUP_CLOSE		"}"
%token					T_GROUP_OPEN		"{"
%token					T_INDENT			"indent"
%token					T_LN				"newline"
%token					T_PAR_BREAK			"paragraph break"
%token					T_COLON				"colon"
%token					T_DOUBLE_COLON		"double-colon"
%token					T_ASSIGNMENT		"<-"
%token <node>			T_INCLUDED_FILE		"file inclusion"
%token <sugar>			T_UNDERSCORE_OPEN	"opening underscore(s)"
%token <simple_sugar>	T_CITATION			"citation"
%token <simple_sugar>	T_LABEL				"label"
%token <simple_sugar>	T_REFERENCE			"reference"
%token <sugar>			T_ASTERISK_OPEN		"opening asterisk(s)"
%token <sugar>			T_BACKTICK_OPEN		"opening backtick"
%token <sugar>			T_BULLET_POINT		"bullet point"
%token <sugar>			T_SEQ_POINT			"sequence point"
%token <sugar>			T_EQUALS_OPEN		"opening equal(s)"
%token <len>			T_UNDERSCORE_CLOSE	"closing underscore(s)"
%token <len>			T_ASTERISK_CLOSE	"closing asterisk(s)"
%token <len>			T_BACKTICK_CLOSE	"closing backtick"
%token <len>			T_EQUALS_CLOSE		"closing equal(s)"
%token <str>			T_DIRECTIVE			"directive"
%token <str>			T_WORD				"word"
%token <str>			T_VARIABLE_REF		"variable"
%token <sugar>			T_HEADING			"heading"

%destructor { dest_unit(&$$); } <doc>
%destructor { if ($$) { dest_free_doc_tree_node($$, false); } } <node>
%destructor { if ($$) { dest_str($$); free($$); } } <str>
%destructor { if ($$) { dest_call_io($$, false), free($$); } } <args>
%destructor { dest_sugar(&$$); } <sugar>
%destructor { dest_simple_sugar(&$$); } <simple_sugar>

%start doc

%{
static void yyerror(YYLTYPE* yyloc, ParserData* params, const char* err);
static Location* alloc_assign_loc(EM_LTYPE yyloc, Str** ifn) __attribute__((malloc));
static void alloc_malloc_error_word(DocTreeNode** out, EM_LTYPE loc, Str** ifn);
static void make_syntactic_sugar_call(DocTreeNode* ret, Sugar sugar, DocTreeNode* arg, Location* loc);
static void make_variable_retrieval(DocTreeNode* node, Str* var, Location* loc);
static void make_variable_assignment(DocTreeNode* node, Str* var, DocTreeNode* val, Location* loc);
static void make_simple_syntactic_sugar_call(DocTreeNode* node, SimpleSugar ssugar, Location* loc);
static void dest_preprocessor_data(PreProcessorData* preproc);

#include "pp/unused.h"
%}

%%

doc : doc_content	{ make_unit(&$$); data->root = $1; }
	;

doc_content
	: %empty								{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_content($$, alloc_assign_loc(@$, data->ifn)); }
	| maybe_par_break_toks file_contents	{ $$ = $2; }

file_contents
	: file_content								{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_content($$, alloc_assign_loc(@$, data->ifn)); prepend_doc_tree_node_child($$, $$->content->content, $1); }
	| file_content par_break_toks file_contents { $$ = $3; prepend_doc_tree_node_child($$, $$->content->content, $1); }
	;

file_content
	: par								{ $$ = $1; $$->flags |= PARAGRAPH_CANDIDATE; }
	| bulleted_list
	| sequenced_list
	| T_INDENT file_contents T_DEDENT	{ $$ = $2; }
	;

par : lines
	;

bulleted_list
	: T_INDENT bullet_list_lines_ne T_DEDENT { $$ = malloc(sizeof(DocTreeNode)); Str* s = malloc(sizeof(Str)); make_strv(s, "ul"); make_doc_tree_node_call($$, s, $2, alloc_assign_loc(@$, data->ifn)); }
	;

bullet_list_lines_ne
	: T_BULLET_POINT file_content bullet_list_lines { $$ = $3; DocTreeNode* call_node = malloc(sizeof(Str)); make_syntactic_sugar_call(call_node, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	;

bullet_list_lines
	: %empty				{ $$ = malloc(sizeof(CallIO)); make_call_io($$); }
	| bullet_list_lines_ne
	;

sequenced_list
	: T_INDENT sequenced_list_lines_ne T_DEDENT	{ $$ = malloc(sizeof(DocTreeNode)); Str* s = malloc(sizeof(Str)); make_strv(s, "ul"); make_doc_tree_node_call($$, s, $2, alloc_assign_loc(@$, data->ifn)); }
	;

sequenced_list_lines_ne
	: T_SEQ_POINT file_content sequenced_list_lines	{ $$ = $3; DocTreeNode* call_node = malloc(sizeof(Str)); make_syntactic_sugar_call(call_node, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	;

sequenced_list_lines
	: %empty					{ $$ = malloc(sizeof(CallIO)); make_call_io($$); }
	| sequenced_list_lines_ne
	;

par_break_toks
	: T_PAR_BREAK maybe_par_break_toks
	;

maybe_par_break_toks
	: %empty
	| T_PAR_BREAK maybe_par_break_toks
	;

lines
	: line										{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_content($$, alloc_assign_loc(@$, data->ifn)); prepend_doc_tree_node_child($$, $$->content->content, $1); $1->parent = $$;}
	| line lines								{ $$ = $2; prepend_doc_tree_node_child($$, $$->content->content, $1); }
	/* | T_INDENT file_contents T_LN T_DEDENT lines { $$ = $2; List* l = malloc(sizeof(List)); iconcat_list(l, $2->content->lines, $4->content->lines); dest_list($2->content->lines, true, NULL); dest_list($4->content->lines, true, NULL); $$->content->lines = l; free($2); free($4); } */
	;

line
	: line_content T_LN
	| T_INCLUDED_FILE
	| T_VARIABLE_REF T_ASSIGNMENT line_content T_LN	{ $$ = malloc(sizeof(DocTreeNode)); make_variable_assignment($$, $1, $3, alloc_assign_loc(@$, data->ifn)); }
	| T_HEADING line_content T_LN					{ $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_DIRECTIVE multi_line_args					{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| error											{ alloc_malloc_error_word(&$$, @$, data->ifn); }
	;

short_args
	: %empty													{ $$ = malloc(sizeof(CallIO)); make_call_io($$); }
	| T_GROUP_OPEN line_content T_GROUP_CLOSE short_args 		{ $$ = $4; prepend_call_io_arg($$, $2); }
	;

line_remainder_args
	: T_GROUP_OPEN line_content T_GROUP_CLOSE line_remainder_args	{ $$ = $4; prepend_call_io_arg($$, $2); }
	| T_COLON line_content_ne										{ $$ = malloc(sizeof(CallIO)); make_call_io($$); prepend_call_io_arg($$, $2); }
	;

multi_line_args
	: T_GROUP_OPEN line_content T_GROUP_CLOSE multi_line_args							{ $$ = $4; prepend_call_io_arg($$, $2); }
	| T_GROUP_OPEN T_LN T_INDENT file_contents T_DEDENT T_GROUP_CLOSE multi_line_args	{ $$ = $7; prepend_call_io_arg($$, $4); }
	| T_COLON T_LN T_INDENT file_contents T_DEDENT trailing_args						{ $$ = $6; prepend_call_io_arg($$, $4); }
	;

trailing_args
	: %empty															{ $$ = malloc(sizeof(CallIO)); make_call_io($$); }
	| T_DOUBLE_COLON T_LN T_INDENT file_contents T_DEDENT trailing_args	{ $$ = $6; prepend_call_io_arg($$, $4); }
	;

line_content
	: %empty			{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_content($$, alloc_assign_loc(@$, data->ifn)); }
	| line_content_ne
	;

line_content_ne
	: line_element line_content			{ $$ = $2; prepend_doc_tree_node_child($$, $$->content->content, $1); }
	| T_DIRECTIVE line_remainder_args	{
											$$ = malloc(sizeof(DocTreeNode));
											make_doc_tree_node_content($$, alloc_assign_loc(@$, data->ifn));
											DocTreeNode* call_node = malloc(sizeof(DocTreeNode));
											make_doc_tree_node_call(call_node, $1, $2, alloc_assign_loc(@$, data->ifn));
											prepend_doc_tree_node_child($$, $$->content->content, call_node);
										}
	;

line_element
	: T_WORD												{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_word($$, $1, alloc_assign_loc(@$, data->ifn)); }
	| T_DIRECTIVE short_args								{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_CITATION											{ $$ = malloc(sizeof(DocTreeNode)); make_simple_syntactic_sugar_call($$, $1, alloc_assign_loc(@$, data->ifn)); }
	| T_REFERENCE											{ $$ = malloc(sizeof(DocTreeNode)); make_simple_syntactic_sugar_call($$, $1, alloc_assign_loc(@$, data->ifn)); }
	| T_LABEL												{ $$ = malloc(sizeof(DocTreeNode)); make_simple_syntactic_sugar_call($$, $1, alloc_assign_loc(@$, data->ifn)); }
	| T_VARIABLE_REF										{ $$ = malloc(sizeof(DocTreeNode)); make_variable_retrieval($$, $1, alloc_assign_loc(@$, data->ifn)); }
	| T_UNDERSCORE_OPEN line_content_ne T_UNDERSCORE_CLOSE	{ ENSURE_MATCHING_PAIR($1, $3, @3, data); $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_ASTERISK_OPEN line_content_ne T_ASTERISK_CLOSE		{ ENSURE_MATCHING_PAIR($1, $3, @3, data); $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_BACKTICK_OPEN line_content_ne T_BACKTICK_CLOSE		{ ENSURE_MATCHING_PAIR($1, $3, @3, data); $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_EQUALS_OPEN line_content_ne T_EQUALS_CLOSE			{ ENSURE_MATCHING_PAIR($1, $3, @3, data); $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	;

%%

static void yyerror(YYLTYPE* yyloc, ParserData* params, const char* err)
{
	++*params->nerrs;
	Location loc = {
		.first_line   = yyloc->first_line,
		.first_column = yyloc->first_column,
		.last_line    = yyloc->last_line,
		.last_column  = yyloc->last_column,
		.src_file     = *params->ifn,
	};
	log_err_at(&loc, "%s", err);
}

static Location* alloc_assign_loc(EM_LTYPE yyloc, Str** ifn)
{
	Location* ret = malloc(sizeof(Location));

	ret->first_line = yyloc.first_line;
	ret->first_column = yyloc.first_column;
	ret->last_line = yyloc.last_line;
	ret->last_column = yyloc.last_column;
	ret->src_file = *ifn;

	return ret;
}

static void alloc_malloc_error_word(DocTreeNode** out, EM_LTYPE loc, Str** ifn)
{
	*out = malloc(sizeof(DocTreeNode));
	Str* erw = malloc(sizeof(Str));
	make_strv(erw, "ERROR");
	make_doc_tree_node_word(*out, erw, alloc_assign_loc(loc, ifn));
}

static void make_syntactic_sugar_call(DocTreeNode* ret, Sugar sugar, DocTreeNode* arg, Location* loc)
{
	CallIO* callio = malloc(sizeof(CallIO));
	make_call_io(callio);
	prepend_call_io_arg(callio, arg);
	make_doc_tree_node_call(ret, sugar.call, callio, loc);
}

static void make_variable_retrieval(DocTreeNode* node, Str* var, Location* loc)
{
	Str* get_call_name = malloc(sizeof(Str));
	make_strv(get_call_name, "get-var");
	CallIO* args = malloc(sizeof(CallIO));
	make_call_io(args);
	DocTreeNode* var_node = malloc(sizeof(DocTreeNode));
	make_doc_tree_node_word(var_node, var, loc);
	prepend_call_io_arg(args, var_node);
	make_doc_tree_node_call(node, get_call_name, args, dup_loc(loc));
}

static void make_variable_assignment(DocTreeNode* node, Str* var, DocTreeNode* val, Location* loc)
{
	Str* set_call_name = malloc(sizeof(Str));
	make_strv(set_call_name, "set-var");
	DocTreeNode* var_node = malloc(sizeof(DocTreeNode));
	make_doc_tree_node_word(var_node, var, loc);
	CallIO* args = malloc(sizeof(CallIO));
	make_call_io(args);
	prepend_call_io_arg(args, val);
	prepend_call_io_arg(args, var_node);
	make_doc_tree_node_call(node, set_call_name, args, dup_loc(loc));
}

static void make_simple_syntactic_sugar_call(DocTreeNode* node, SimpleSugar ssugar, Location* loc)
{
	CallIO* io = malloc(sizeof(CallIO));
	make_call_io(io);
	DocTreeNode* arg_node = malloc(sizeof(DocTreeNode));
	make_doc_tree_node_word (arg_node, ssugar.arg, loc);
	prepend_call_io_arg(io, arg_node);
	make_doc_tree_node_call(node, ssugar.call, io, dup_loc(loc));
}

static void dest_preprocessor_data(PreProcessorData* preproc)
{
	UNUSED(preproc);
}

unsigned int parse_file(Maybe* mo, Locked* mtNamesList, Args* args, char* fname)
{
	log_info("Parsing file '%s'", fname);
	bool use_stdin = !strcmp(fname, "-");
	Str* ifn = malloc(sizeof(Str));
	make_strv(ifn, use_stdin ? "(stdin)" : fname);
	FILE* fp = use_stdin ? stdin : fopen(ifn->str, "r");
	if (!fp)
	{
		if (!strrchr(ifn->str, '.'))
		{
			char* fname_with_extension = malloc(1 + ifn->len + sizeof(DEFAULT_CONTENT_EXTENSION));
			strcpy(fname_with_extension, ifn->str);
			strcpy(fname_with_extension + ifn->len, DEFAULT_CONTENT_EXTENSION);
			dest_str(ifn);
			make_strr(ifn, fname_with_extension);

			if (!(fp = fopen(ifn->str, "r")))
			{
				log_err("Failed to open file either '%s' or '%s'", fname, ifn->str);
				make_maybe_nothing(mo);
				return 1;
			}
		}
		else
		{
			log_err("Failed to open file '%s'", fname);
			make_maybe_nothing(mo);
			return 1;
		}
	}

	log_debug("Opened file '%s'", fname);

	USE_LOCK(List* namesList, mtNamesList, append_list(namesList, ifn));

	int nerrs = 0;
	LexerData ld = {
		.comment_lvl = 0,
		.indent_lvl = 0,
		.indent_lvl_target = 0,
		.tab_size = args->tab_size,
		.opening_emph = true,
		.nerrs = &nerrs,
		.ifn = ifn,
		.ifp = fp,
		.mtNamesList = mtNamesList,
		.args = args,
		.preproc = { 0 },
	};
	yyscan_t scanner;
	em_lex_init(&scanner);
	em_set_extra(&ld, scanner);
	em_set_in(fp, scanner);
	ParserData pd = {
		.args = args,
		.root = NULL,
		.ifn = &ld.ifn,
		.nerrs = &nerrs,
		.scanner = scanner,
	};

	log_debug("Starting parser on file '%s'", ifn->str);
	em_parse(&pd);
	em_lex_destroy(scanner);
	if (!use_stdin)
		fclose(fp);

	if (!nerrs && pd.root)
		make_maybe_just(mo, pd.root);
	else
	{
		make_maybe_nothing(mo);
	}

	dest_preprocessor_data(&ld.preproc);

	return nerrs;
}
