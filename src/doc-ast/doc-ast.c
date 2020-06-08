#include "doc-ast.h"

#include <stdlib.h>

DocAst* create_doc_ast_node(ANType antype, DocAst* nxt, DocAst* prev, DocAst* pnt, size_t len, ANData andata)
{
	DocAst* docAst = calloc(1, sizeof(DocAst));
	docAst->antype = antype;
	docAst->nxt	   = nxt;
	docAst->prev   = prev;
	docAst->pnt	   = pnt;
	docAst->len	   = len;
	docAst->andata = andata;
	return docAst;
}

void delete_doc_ast_node(DocAst* docAst)
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
				delete_doc_ast_node(arg);
				arg = arg->nxt;
			}
			break;
		default:
			fprintf(stderr, "Attempted to delete memory of DocAst node of type %d\n", docAst->antype);
	}
}
