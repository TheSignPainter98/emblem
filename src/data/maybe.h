#pragma once

#include "pp/lambda.h"
#include "pp/not_implemented.h"
#include "config.h"
#include "destructor.h"
#include "fmap.h"
#include "unit.h"
#include <stdbool.h>

/**
 * @brief Defines types which may or may not have a value
 */
typedef struct
{
	enum
	{
		NOTHING,
		JUST
	} type;
	union
	{
		Unit nothing;
		void* just;
	};
} Maybe;

/**
 * @brief Construct a maybe-type object with the nothing constructor
 *
 * @param m Pointer to a location to initialise
 */
void make_maybe_nothing(Maybe* m);

/**
 * @brief Construct a maybe-type object with the just constructor
 *
 * @param m Pointer to a location to initialise
 * @param data Data to store in the just
 */
void make_maybe_just(Maybe* m, void* data);

/**
 * @brief Destroy a maybe-type object. Any stored data must be destroyed separately.
 *
 * @param m Pointer to a meybe object to destroy
 * @param ed Element destructor to be called on the just field if the constructor permits
 */
void dest_maybe(Maybe* m, Destructor ed);

/**
 * @brief Apply a function to the stored data in the maybe and output a new maybe object with the new value.
 *
 * If `mi` is `NOTHING`, then `mo` will be `NOTHING`.
 * Otherwise, if `mi` is `JUST x`, then `mo` will be `JUST f(x)`
 *
 * @param mo Ouptut maybe object. Should be an uninitialised maybe-type pointer.
 * @param mi Input maybe object which will have `f` applied to it
 * @param f Function to apply to any data inside `mi`
 */
void fmap_maybe(Maybe* restrict mo, Maybe* restrict mi, Fmap f);

/**
 * @brief Check whether a maybe-type object represents a success
 *
 * @param m Pointer to a maybe object
 *
 * @return Returns true if the constructor of `m` is Just, otherwise false
 */
bool succ_maybe(Maybe* m) __attribute__((pure));
