#pragma once

/**
 * @brief Type of a function which can be used in an fmap operation
 *
 * @param o The value outputted by the fmap
 * @param i The unput value
 */
typedef void (*Fmap)(void** o, void* i);
