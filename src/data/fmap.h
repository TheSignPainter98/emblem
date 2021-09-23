/**
 * @file fmap.h
 * @brief Provides type definition for fmap functions, which allow computations on data _within_ structures
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

/**
 * @brief Type of a function which can be used in an fmap operation
 *
 * @param o The value outputted by the fmap
 * @param i The unput value
 */
typedef void (*Fmap)(void** o, void* i);
