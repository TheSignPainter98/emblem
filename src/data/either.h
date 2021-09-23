/**
 * @file either.h
 * @brief Exposes functions for processing Either objects, which can represent values of two types
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "config.h"
#include "destructor.h"
#include "fmap.h"
#include "pp/lambda.h"
#include "unit.h"
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

/**
 * @brief Fmap over either-type object
 *
 * If the type of `ei` is LEFT, then the type of `eo` will also be left and `eo.left == ei.left`.
 * Otherwise, the type of `eo` is RIGHT and the value of `eo.right = f(ei.right)`.
 *
 * @param eo Output either type
 * @param ei Input Either type
 * @param f Function to apply to the contents of `ei` to produce `eo`
 */
void fmap_either(Either* eo, Either* ei, Fmap f);

/**
 * @brief Destroy an either-type object.
 *
 * @param e Pointer to the either object to destroy
 * @param led Element destructor for the left
 * @param red Element destructor for the right
 */
void dest_either(Either* e, Destructor led, Destructor red);

/**
 * @brief Return whether a given either-type object represents a successful result
 *
 * @param e Pointer to the either object to check
 *
 * @return true if e uses the right constructor otherwise false
 */
bool succ_either(Either* e);
