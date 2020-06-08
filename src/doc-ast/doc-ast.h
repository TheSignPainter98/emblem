#ifndef DOC_AST_H_
#define DOC_AST_H_

#include <stdio.h>

/**
 * @brief DocAst node type
 */
typedef enum ANType_e
{
	/**
	 * @brief DocAst node represents a word
	 */
	WORD,
	/**
	 * @brief DocAst node represents punctuation
	 */
	PUNCT,
	/**
	 * @brief DocAst node represents a horizontal gap (e.g. word space)
	 */
	HGAP,
	/**
	 * @brief DocAst node represents a vertical gap (e.g. paragraph skip)
	 */
	VGAP,
	/**
	 * @brief DocAst node represents a function call
	 */
	CALL
} ANType;

/**
 * @brief Doc AST node data
 */
typedef union ANData_e
{
	/**
	 * @brief Word data
	 */
	struct {
		/**
		 * @brief Word content
		 */
		char* wrd;
		/**
		 * @brief Word content character length
		 */
		size_t wlen;
	} word;
	/**
	 * @brief Punctuation data
	 */
	struct {
		/**
		 * @brief Punctuation content
		 */
		char* pnct;
		/**
		 * @brief Punctuation content length
		 */
		size_t plen;
	} punct;
	/**
	 * @brief Horizontal gap data
	 */
	struct {
		/**
		 * @brief Horizontal gap content
		 */
		char* hgp;
		/**
		 * @brief Horizontal gap character length
		 */
		size_t hlen;
	} hgap;
	/**
	 * @brief Vertical gap data
	 */
	struct {
		/**
		 * @brief Vertical gap content
		 */
		char* vgp;
		/**
		 * @brief Vertical gap character length
		 */
		size_t vlen;
	} vgap;
	/**
	 * @brief Function call data
	 */
	struct {
		/**
		 * @brief Name of function being called
		 */
		const char* fname;
		/**
		 * @brief Source package of function being called
		 */
		const char* fpkg;
		/**
		 * @brief Pointer to function being called
		 *
		 * @param The parameter list of the function
		 */
		int (*fptr)(struct DocAst_s*);
		/**
		 * @brief Function call parameter list
		 */
		struct DocAst_s* argList;
	} call;
} ANData;

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

#endif /* DOC_AST_H_ */
