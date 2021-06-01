#pragma once

/**
 * @brief Valid values returned by a comparison.
 */
typedef enum
{
	/**
	 * @brief Less than
	 */
	CMP_LT = -1,
	/**
	 * @brief Equal
	 */
	CMP_EQ = 0,
	/**
	 * @brief Greater than
	 */
	CMP_GT = 1
} Cmp;

/**
 * @brief Type of a function which compares two values
 *
 * @param v1 A value to compare
 * @param v2 Another value to compare
 *
 * @return The comparison result of `v1` against `v2`
 */
typedef Cmp (*Comparator)(void* v1, void* v2);

/**
 * @brief The signature of a comparator
 *
 * @param name The name of the comparison functino
 *
 * @return The signature of the comparison function for `name`s
 */
#define CMP_SIG(name) Cmp cmp_##name##s(void* v1, void* v2)

/**
 * @brief Compare characters
 *
 * @param v1 A character to compare
 * @param v2 Another character to compare
 */
CMP_SIG(char);
/**
 * @brief Compare doubles
 *
 * @param v1 A double to compare
 * @param v2 Another double to compare
 */
CMP_SIG(double);
/**
 * @brief Compare float
 *
 * @param v1 A float to compare
 * @param v2 Another float to compare
 */
CMP_SIG(float);
/**
 * @brief Compare ints
 *
 * @param v1 An int to compare
 * @param v2 Another int to compare
 */
CMP_SIG(int);
/**
 * @brief Compare size_ts
 *
 * @param v1 A size_t to compare
 * @param v2 Another size_t to compare
 */
CMP_SIG(size_t);
/**
 * @brief Compare void pointer. This compares the numerical value of the pointers, not their content!
 *
 * @param v1 A void pointer to compare
 * @param v2 Another void pointer to compare
 */
CMP_SIG(ptr);
/**
 * @brief Compare strings
 *
 * @param v1 A string to compare
 * @param v2 Another string to compare
 */
CMP_SIG(str);

/**
 * @brief Check if two C-strings are equal (to improve readability with strcmp)
 *
 * @param s Pointer to a string to check
 * @param t Pointer to a string to check
 *
 * @return true if the strings are equal, false otherwise
 */
bool streq(char const* s, char const* t);
