#ifndef AST_NODE_DATA_H_
#define AST_NODE_DATA_H_

#include "../measurement/measurement.h"
#include <stdio.h>

/**
 * @brief DocAst node type
 */
typedef enum
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
	CALL,
	/**
	 * @brief DocAst node represents a floater
	 */
	FLOATER
} ANType;

/**
 * @brief Word data
 */
typedef struct
{
	/**
	 * @brief Word content
	 */
	char* wrd;
	/**
	 * @brief Word content character length
	 */
	size_t wlen;
} WordData;

/**
 * @brief Punctuation data
 */
typedef struct
{
	/**
	 * @brief Punctuation content
	 */
	char* pnct;
	/**
	 * @brief Punctuation content length
	 */
	size_t plen;
} PunctData;

/**
 * @brief Horizontal gap data
 */
typedef struct
{
	/**
	 * @brief Horizontal gap content
	 */
	char* hgp;
	/**
	 * @brief Horizontal gap character length
	 */
	size_t hlen;
} HGapData;

/**
 * @brief Vertical gap data
 */
typedef struct
{
	/**
	 * @brief Vertical gap content
	 */
	char* vgp;
	/**
	 * @brief Vertical gap character length
	 */
	size_t vlen;
} VGapData;

/**
 * @brief Function call data
 */
typedef struct
{
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
} CallData;

/**
 * @brief Floater should be placed to the East
 */
#define FLOATER_LOG_HINT_EAST 0
/**
 * @brief Floater should be palced to the North
 */
#define FLOATER_LOG_HINT_NORTH 0.25
/**
 * @brief Floater should be placed to the West
 */
#define FLOATER_LOG_HINT_WEST 0.5
/**
 * @brief Floater should be placed to the South
 */
#define FLOATER_LOG_HINT_SOUTH 0.75

/**
 * @brief Hint for the direction in which a floater should be placed
 */
typedef struct
{
	/**
	 * @brief Floater is indifferent to the direction it is placed.
	 *
	 * The angle value must be ignored if and only if indiff == false
	 */
	bool indiff;
	/**
	 * @brief Direction the floater would like to be placed in, value is in the interval [0,1)
	 *
	 * Angle ranges from zero to less than one, representing the fraction of a revolution completed anti-clockwise from east
	 */
	double angle;
} FloaterLocHint;

/**
 * @brief Hint location for float placement
 */
typedef Pos FloatLocHintCoords;

/**
 * @brief Floater data
 */
typedef struct
{
	/**
	 * @brief Preferred direction to place the floater
	 */
	FloaterLocHint directionHint;
	/**
	 * @brief Preferred location of the floater
	 */
	FloatLocHintCoords* coordsHint;
	/**
	 * @brief coordsHint is absolute (based on the page) as opposed to relative (from the current position)
	 */
	bool absoluteCoordsHint;
	/**
	 * @brief Priority of the floater placement
	 */
	double locPriority;
	/**
	 * @brief Content of the floater
	 */
	struct DocAst_s* cnt;
} FloaterData;

/**
 * @brief Doc AST node data
 */
typedef union ANData_e
{
	/**
	 * @brief Word data
	 */
	WordData word;
	/**
	 * @brief Punctuation data
	 */
	PunctData punct;
	/**
	 * @brief Horizontal gap data
	 */
	HGapData hgap;
	/**
	 * @brief Vertical gap data
	 */
	VGapData vgap;
	/**
	 * @brief Call data
	 */
	CallData call;
	/**
	 * @brief Floater data
	 */
	FloaterData floater;
} ANData;

#endif /* AST_NODE_DATA_H_ */
