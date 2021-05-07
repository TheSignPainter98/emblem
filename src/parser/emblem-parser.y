%code requires {
extern int em_lex();
#include "argp.h"
#include "parser.h"
#include "data/locked.h"
void parse_file(Maybe* eo, Locked* namesList, Args* args, char* fname);

typedef struct
{
	int comment_lvl;
	int indent_lvl;
	int indent_lvl_target;
	int tab_size;
	bool opening_emph;
	int* nerrs;
	FILE* ifp;
	Str* ifn;
} LexerData;

typedef struct
{
	Args* args;
	Doc* doc;
	Str* ifn;
	int* nerrs;
	void** scanner;
} ParserData;
}

%{
#include "pp/unused.h"

#include "data/str.h"
#include "data/either.h"
#include "em-emblem-parser.h"
#include "logs/logs.h"
#include "em-emblem-lexer.h"
#include <stdbool.h>
#include <string.h>

#define YYLEX_PARAM_ data->scanner
%}

%define 		parse.trace true
%define 		parse.error verbose
%define 		api.pure    full
%define 		api.prefix  {em_}
%define 		lr.type 	ielr
%parse-param 	            { ParserData* data }
%lex-param 		            { yyscan_t YYLEX_PARAM_ }
%locations
%expect 0

%union {
	Unit doc;
	DocTreeNode* node;
	CallIO* args;
	Str* str;
	Str* sugar;
}

%nterm <args>			args
%nterm <doc>  			doc
%nterm <node>			line
%nterm <node>			line_content
%nterm <node>			line_content_ne
%nterm <node>			line_element
%nterm <node>			lines
%nterm <node>			maybe_lines
%token 					T_DEDENT			"dedent"
%token 					T_GROUP_CLOSE		"}"
%token 					T_GROUP_OPEN		"{"
%token 					T_INDENT 			"indent"
%token 					T_LN 				"newline"
%token 		  			T_COLON				"colon"
%token <sugar>			T_UNDERSCORE_OPEN 	"opening underscore(s)"
%token <sugar>			T_ASTERISK_OPEN		"opening asterisk(s)"
%token <sugar>			T_BACKTICK_OPEN		"opening backtick"
%token <sugar>			T_EQUALS_OPEN		"opening equal(s)"
%token 					T_UNDERSCORE_CLOSE	"closing underscore(s)"
%token 					T_ASTERISK_CLOSE	"closing asterisk(s)"
%token 					T_BACKTICK_CLOSE	"closing backtick"
%token 					T_EQUALS_CLOSE		"closing equal(s)"
%token <str> 			T_DIRECTIVE			"directive"
%token <str> 			T_WORD 				"word"
%token <sugar> 			T_HEADING			"heading"

%destructor { dest_unit(&$$); } <doc>
%destructor { if ($$) { dest_free_doc_tree_node($$, false); } } <node>
%destructor { if ($$) { dest_str($$); free($$); } } <str>
%destructor { if ($$) { dest_call_io($$, false), free($$); } } <args>
%destructor { if ($$) { free($$); } } <sugar>

%start doc

%{
static void yyerror(YYLTYPE* yyloc, ParserData* params, const char* err);
static Location* alloc_assign_loc(EM_LTYPE yyloc, Str* ifn) __attribute__((malloc));
static void alloc_malloc_error_word(DocTreeNode** out, EM_LTYPE loc, Str* ifn);
static void make_syntactic_sugar_call(DocTreeNode* ret, Str* call, DocTreeNode* arg, Location* loc);
%}

%%

doc : maybe_lines	{ make_unit(&$$); data->doc = malloc(sizeof(Doc)); make_doc(data->doc, $1, data->args); }
	;

maybe_lines
	: %empty									{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_lines($$, alloc_assign_loc(@$, data->ifn)); }
	| lines										{ $$ = $1; }
	;

lines
	: line										{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_lines($$, alloc_assign_loc(@$, data->ifn)); prepend_doc_tree_node_child($$, $$->content->lines, $1); $1->parent = $$;}
	| line lines								{ $$ = $2; prepend_doc_tree_node_child($$, $$->content->lines, $1); }
	/* | T_INDENT maybe_lines T_DEDENT maybe_lines	{ $$ = $2; List* l = malloc(sizeof(List)); concat_list(l, $2->content->lines, $4->content->lines); dest_list($2->content->lines, true, NULL); dest_list($4->content->lines, true, NULL); $$->content->lines = l; free($2); free($4); } */
	;

line
	: line_content T_LN  			{ $$ = $1; }
	| T_HEADING line_content T_LN	{ $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_DIRECTIVE args 				{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| error 						{ alloc_malloc_error_word(&$$, @$, data->ifn); }
	;

args
	: %empty										{ $$ = malloc(sizeof(CallIO)); make_call_io($$); }
	| T_COLON line_content_ne T_LN					{ $$ = malloc(sizeof(CallIO)); make_call_io($$); prepend_call_io_arg($$, $2); }
	| T_COLON T_LN T_INDENT maybe_lines T_DEDENT	{ $$ = malloc(sizeof(CallIO)); make_call_io($$); prepend_call_io_arg($$, $4); }
	| T_GROUP_OPEN line_content T_GROUP_CLOSE args 	{ $$ = $4; prepend_call_io_arg($$, $2); }
	;

line_content
	: %empty												{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_line($$, alloc_assign_loc(@$, data->ifn)); }
	| line_content_ne										{ $$ = $1; }
	;

line_content_ne
	: line_element line_content								{ $$ = $2; prepend_doc_tree_node_child($$, $$->content->line, $1); }
	;

line_element
	: T_WORD												{ $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_word($$, $1, alloc_assign_loc(@$, data->ifn)); }
	| T_UNDERSCORE_OPEN line_content_ne T_UNDERSCORE_CLOSE	{ $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_ASTERISK_OPEN line_content_ne T_ASTERISK_CLOSE		{ $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_BACKTICK_OPEN line_content_ne T_BACKTICK_CLOSE		{ $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	| T_EQUALS_OPEN line_content_ne T_EQUALS_CLOSE			{ $$ = malloc(sizeof(DocTreeNode)); make_syntactic_sugar_call($$, $1, $2, alloc_assign_loc(@$, data->ifn)); }
	;


/* doc */
	/* : paragraphs T_LN 				{ log_debug("Document was full of paragraphs"); } */
	/* | paragraph { log_warn("ONly paragraph"); } */
	/* | error							{ log_warn("There was an error somewhere in the document"); } */
	/* ; */

/* paragraphs */
	/* : paragraph						{ log_debug("Starting a par list"); } */
	/* | paragraph T_LN T_LN paragraphs 	{ log_debug("Continuing a par list"); } */
	/* ; */

/* paragraph */
	/* : %empty					{ log_debug("Paragraph  empty"); } */
	/* | par_part 					{ log_debug("Starting a par"); } */
	/* | par_part T_LN paragraph 	{ log_debug("Continuing a par"); } */
	/* ; */

/* par_part */
	/* : sentence */
	/* | T_DIRECTIVE T_COLON T_LN T_INDENT paragraphs T_LN T_DEDENT { log_debug("Got directive '%s'", $1->str); } */
	/* ; */

/* sentence */
	/* : sentence_part				{ log_debug("Starting a sentence"); } */
	/* | sentence_part sentence 	{ log_debug("Continuing a sentence"); } */
	/* ; */

/* sentence_part */
	/* : T_WORD	  { log_debug("Got word '%s'", $1->str); $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_word($$, $1, alloc_assign_loc(@$, data->ifn)); } */
	/* | T_COLON	  { log_debug("Got colon ':'"); $$ = malloc(sizeof(DocTreeNode)); Str* s = malloc(sizeof(Str)); make_strc(s, ":"); make_doc_tree_node_word($$, s, alloc_assign_loc(@$, data->ifn)); } */
	/* | T_DIRECTIVE { log_debug("Got directive '%s'", $1->str); $$ = malloc(sizeof(DocTreeNode)); make_doc_tree_node_word($$, $1, alloc_assign_loc(@$, data->ifn)); } */
	/* ; */

%%

static void yyerror(YYLTYPE* yyloc, ParserData* params, const char* err)
{
	++*params->nerrs;
	log_err("%s:%d:%d: %s", params->ifn->str, yyloc->first_line, yyloc->first_column, err);
}

static Location* alloc_assign_loc(EM_LTYPE yyloc, Str* ifn)
{
	Location* ret = malloc(sizeof(Location));

	ret->first_line = yyloc.first_line;
	ret->first_column = yyloc.first_column;
	ret->last_line = yyloc.last_line;
	ret->last_column = yyloc.last_column;
	ret->src_file = ifn;

	return ret;
}

static void alloc_malloc_error_word(DocTreeNode** out, EM_LTYPE loc, Str* ifn)
{
	*out = malloc(sizeof(DocTreeNode));
	Str* erw = malloc(sizeof(Str));
	make_strv(erw, "ERROR");
	make_doc_tree_node_word(*out, erw, alloc_assign_loc(loc, ifn));
}

static void make_syntactic_sugar_call(DocTreeNode* ret, Str* call, DocTreeNode* arg, Location* loc)
{
	CallIO* callio = malloc(sizeof(CallIO));
	make_call_io(callio);
	prepend_call_io_arg(callio, arg);
	make_doc_tree_node_call(ret, call, callio, loc);
}

void parse_file(Maybe* mo, Locked* mtNamesList, Args* args, char* fname)
{
	log_info("Parsing file '%s'", fname);
	bool use_stdin = !strcmp(fname, "-");
	FILE* fp = use_stdin ? stdin : fopen(fname, "r");
	if (!fp)
	{
		log_err("Failed to open file '%s'", fname);
		make_maybe_nothing(mo);
		return;
	}
	else
		log_debug("Opened file '%s'", fname);

	Str* ifn = malloc(sizeof(Str));
	make_strv(ifn, use_stdin ? "(stdin)" : fname);
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, ifn);
	USE_LOCK(List* namesList, mtNamesList, append_list_node(namesList, ln));

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
	};
	yyscan_t scanner;
	em_lex_init(&scanner);
	em_set_extra(&ld, scanner);
	em_set_in(fp, scanner);
	ParserData pd = {
		.args = args,
		.doc = NULL,
		.ifn = ifn,
		.nerrs = &nerrs,
		.scanner = scanner,
	};

	log_debug("Starting parser on file '%s'", pd.ifn->str);
	em_parse(&pd);
	em_lex_destroy(scanner);
	if (!use_stdin)
		fclose(fp);


	if (!nerrs && pd.doc)
		make_maybe_just(mo, pd.doc);
	else
	{
		log_err("Parsing file '%s' failed with %d error%s.", fname, nerrs, nerrs - 1 ? "s" : "");
		dest_str(ifn);
		free(ifn);
		make_maybe_nothing(mo);
	}
}
