#include "../../src/doc-struct/doc-ast.h"

#include <criterion/criterion.h>
#include <string.h>

Test(chec_ast_checks, correct_node_memory_cycle)
{
	DocAst docAst, *nxt, *prev, *pnt;
	nxt = (DocAst*)0xdeadbeef;
	prev = (DocAst*)0xfeedcafe;
	pnt = (DocAst*)0xadadabbd;
	AANData an;
	an.word.wrd = strdup("");
	an.word.wlen = 0;

	int rc = doc_ast_node_create(&docAst, AWORD, nxt, prev, pnt, 0, an);
	cr_assert(rc == 0, "Failed to create new DocAst node\n");
	cr_assert(docAst.antype == AWORD, "docAst type incorrectly set\n");
	cr_assert(docAst.nxt == nxt, "docAst type incorrectly set\n");
	cr_assert(docAst.prev == prev, "docAst type incorrectly set\n");
	cr_assert(docAst.pnt == pnt, "docAst type incorrectly set\n");
	cr_assert(docAst.len == 0, "docAst type incorrectly set\n");
	cr_assert(!strcmp(docAst.andata.word.wrd, an.word.wrd), "docAst doesn't contain the same string value as it was given\n");
	cr_assert(docAst.andata.word.wrd != an.word.wrd, "docAst didn't copy the word it was given");
	cr_assert(docAst.andata.word.wlen == an.word.wlen, "docAst type incorrectly set\n");

	doc_ast_node_destroy(&docAst);
}
