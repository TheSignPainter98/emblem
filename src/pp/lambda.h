#pragma once

#include "ignore_warning.h"

#if __GNUC__
extern int lambda_; // This doesn't exist, don't use it, it's just to shut up the syntax highlighter
/**
 * @brief Create an impure anonymous function
 *
 * @param r Return type of the lambda expression
 * @param ps Paramters of the lambda expression
 * @param b Body of the lambda expression, must be surrounded by curly braces
 *
 * @return A pointer to an anonymous function with parameters `ps`, return-type `r` and body `b`
 */
#	define ilambda(r, ps, b)                                                                                          \
		({                                                                                                             \
			r lambda_ ps b;                                                                                            \
			lambda_;                                                                                                   \
		})
/**
 * @brief Create a pure anonymous function
 *
 * @param r Return type of the lambda, cannot be void
 * @param ps Parameters of the lambda
 * @param e Expression of the body of the lambda, must have non-void type
 *
 * @return A pointer to an anonymous function which takes `ps` and returns the value of `e` of type `r`
 */
#	define lambda(r, ps, e)                                                                                           \
		({                                                                                                             \
			r lambda_ ps { return (e); };                                                                              \
			lambda_;                                                                                                   \
		})
#else
#	error "GCC extensions are required to compile lambda expressions"
#endif

/**
 * @brief A named function signature
 *
 * Useful for concisely writing the types of parameter function-pointers
 *
 * @param r Return type of the lambda, cannot be void
 * @param n Expression of the body of the lambda, must have non-void type
 * @param ps Parameters of the lambda
 *
 * @return The signature of a function-pointer of name `n` which takes `ps` and returns `r`
 */
#define func_sig(r, n, ps) r(*n) ps

/**
 * @brief A function type
 *
 * Useful for concisely writing the types of parameter function-pointers
 *
 * @param r Return type of the function signature
 * @param ps Parameters of the function
 *
 * @return The type of a function-pointer which takes `ps` and returns `r`
 */
#define func_type(r, ps) r(*) ps
