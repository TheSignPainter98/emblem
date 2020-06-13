#include "doc-ast.h"

#include <stdlib.h>
#include <string.h>

int doc_ast_node_create(DocAst* docAst, ANType antype, DocAst* nxt, DocAst* prev, DocAst* pnt, size_t len, AANData an)
{
	docAst->antype = antype;
	docAst->nxt	   = nxt;
	docAst->prev   = prev;
	docAst->pnt	   = pnt;
	docAst->len	   = len;
	/* docAst->andata = andata; */
	switch (docAst->antype)
	{
		case AWORD:
			docAst->andata.word.wrd	 = strdup(an.word.wrd);
			docAst->andata.word.wlen = an.word.wlen;
			break;
		case APUNCT:
			docAst->andata.punct.pnct = strdup(an.punct.pnct);
			docAst->andata.punct.plen = an.punct.plen;
			break;
		case AHGAP:
			docAst->andata.hgap.hgp	 = strdup(an.hgap.hgp);
			docAst->andata.hgap.hlen = an.hgap.hlen;
			break;
		case AVGAP:
			docAst->andata.vgap.vgp = strdup(an.vgap.vgp);
			docAst->andata.vgap.vlen = an.vgap.vlen;
			break;
		case ACALL:
			docAst->andata.call.fname = strdup(an.call.fname);
			docAst->andata.call.fpkg = strdup(an.call.fpkg);
			docAst->andata.call.argList = an.call.argList;
			break;
		case AFLOATER:
			docAst->andata.floater.locPriority = an.floater.locPriority;
			docAst->andata.floater.cnt = an.floater.cnt;
			memcpy(docAst->andata.floater.coordsHint, an.floater.coordsHint, sizeof(AFloatLocHintCoords));
			memcpy(docAst->andata.floater.directionHint, an.floater.directionHint, sizeof(AFloaterLocHint));
			break;
		default:
			fprintf(stderr, "Unknown andata type %d\n",  docAst->antype);
			exit(1);
	}
	return 0;
}

void doc_ast_node_destroy(DocAst* docAst)
{
	switch (docAst->antype)
	{
		case AWORD:
			free(docAst->andata.word.wrd);
			break;
		case APUNCT:
			free(docAst->andata.punct.pnct);
			break;
		case AHGAP:
			free(docAst->andata.hgap.hgp);
			break;
		case AVGAP:
			free(docAst->andata.vgap.vgp);
			break;
		case ACALL:
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
		case AFLOATER:
			free(docAst->andata.floater.directionHint);
			free(docAst->andata.floater.coordsHint);
			doc_ast_node_destroy(docAst->andata.floater.cnt);
		default:
			fprintf(stderr, "Attempted to delete memory of DocAst node of type %d\n", docAst->antype);
			exit(1);
	}
}
