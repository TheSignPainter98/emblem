#ifndef DOC_AST_H_
#define DOC_AST_H_

#include <stdio.h>
#include "ast-node-data.h"

/**
 * @brief Document abstract syntax tree structure
 */
typedef struct DocAst_s
{
	/**
	 * @brief Doc AST node type
	 */
	ANType antype;
	/**
	 * @brief Pointer to the next node in the doc AST
	 */
	struct DocAst_s* nxt;
	/**
	 * @brief Pointer to the previous node in the doc AST
	 */
	struct DocAst_s* prev;
	/**
	 * @brief Pointer to the parent node in the doc AST
	 */
	struct DocAst_s* pnt;
	/**
	 * @brief Character length of the contents of this doc AST node and its children
	 */
	size_t len;
	/**
	 * @brief Node data
	 */
	ANData andata;
} DocAst;

/**
 * @brief Prepare memory for a document AST node
 *
 * @param antype Doc AST node type
 * @param nxt Pointer to the next node in the AST or NULL
 * @param prev Pointer to the previous AST node or NULL
 * @param pnt Pointer to the parent AST node or NULL
 * @param len Number of characters present in this node and any children
 * @param andata AST node data
 *
 * @return A pointer to a new DocAst node
 */
DocAst* create_doc_ast_node(ANType antype, DocAst* nxt, DocAst* prev, DocAst* pnt, size_t len, ANData andata);
/**
 * @brief Free the memory of a DocAst node and any children
 *
 * @param docAst The node to delete
 */
void delete_doc_ast_node(DocAst* docAst);

#endif /* DOC_AST_H_ */
