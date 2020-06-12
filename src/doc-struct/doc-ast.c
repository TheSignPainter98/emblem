#include "doc-ast.h"

#include <stdlib.h>

int doc_ast_node_create(DocAst* docAst, ANType antype, DocAst* nxt, DocAst* prev, DocAst* pnt, size_t len, ANData andata)
{
	docAst->antype = antype;
	docAst->nxt	   = nxt;
	docAst->prev   = prev;
	docAst->pnt	   = pnt;
	docAst->len	   = len;
	docAst->andata = andata;
	return 0;
}

void doc_ast_node_destroy(DocAst* docAst)
{
	switch (docAst->antype)
	{
		case WORD:
			free(docAst->andata.word.wrd);
			break;
		case PUNCT:
			free(docAst->andata.punct.pnct);
			break;
		case HGAP:
			free(docAst->andata.hgap.hgp);
			break;
		case VGAP:
			free(docAst->andata.vgap.vgp);
			break;
		case CALL:
			free((char*)docAst->andata.call.fname);
			free((char*)docAst->andata.call.fpkg);

			// Recursively free children
			DocAst* arg = docAst->andata.call.argList;
			while (arg)
			{
				doc_ast_node_destroy(arg);
				arg = arg->nxt;
			}
			break;
		default:
			fprintf(stderr, "Attempted to delete memory of DocAst node of type %d\n", docAst->antype);
	}
}
