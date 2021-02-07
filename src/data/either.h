#pragma once

#include "unit.h"
#include "config.h"
#include "pp/lambda.h"
#include <stdbool.h>

typedef struct
{
	enum
	{
		LEFT,
		RIGHT
	} type;
	union
	{
		void* left;
		void* right;
	};
} Either;

/**
 * @brief Construct an either-type object with the left constructor
 *
 * @param e Pointer to the either object to initialise
 * @param left_val Value to place into the left constructor
 */
void make_either_left(Either* e, void* left_val);

/**
 * @brief Construct an either-type object with the right constructor
 *
 * @param e Pointer to the either object to initialise
 * @param right_val Value to place into the right constructor
 */
void make_either_right(Either* e, void* right_val);

void fmap_either(Either* eo, Either* ei, func_sig(void, f, (void**, void*)));

/**
 * @brief Destroy an either-type object.
 *
 * @param e Pointer to the either object to destroy
 */
void dest_either(Either* e);

/**
 * @brief Return whether a given either-type object represents a successful result
 *
 * @param e Pointer to the either object to check
 *
 * @return true if e uses the right constructor otherwise false
 */
bool succ_either(Either* e);
