#pragma once

#include "argp.h"
#include "config.h"
#include "doc-struct/location.h"
#include <stdarg.h>

/**
 * @brief Initialise logging
 */
void init_logs(Args* args);

/**
 * @brief Finalise logging
 */
void fini_logs(void);

/**
 * @brief Write a warning to stderr
 *
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
int log_warn(const char* restrict format, ...) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write a warning to stderr, using a va_list of format-arguments
 *
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
int vlog_warn(const char* restrict format, va_list va);
/**
 * @brief Write a warning to stderr referencing a source location
 *
 * @param loc The source location to reference
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
int log_warn_at(Location* loc, const char* restrict format, ...);
/**
 * @brief Write a warning to stderr referencing a source location, using a va_list of format-arguments
 *
 * @param loc The source location to reference
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
int vlog_warn_at(Location* loc, const char* restrict format, va_list va);
/**
 * @brief Write an error stderr
 *
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void log_err(const char* restrict format, ...) __attribute__((cold)) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write an error stderr, using a va_list of format-arguments
 *
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void vlog_err(const char* restrict format, va_list va);
/**
 * @brief Write an error stderr referencing a source location
 *
 * @param loc The source location to reference
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void log_err_at(Location* loc, const char* restrict format, ...) __attribute__((cold))
__attribute__((format(printf, 2, 3)));
/**
 * @brief Write an error stderr referencing a source location, using a va_list of format-arguments
 *
 * @param loc The source location to reference
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void vlog_err_at(Location* loc, const char* restrict format, va_list va);
/**
 * @brief Write information to stderr
 *
 * @param format Information format (printf)
 * @param ... Possible printf arguments
 */
void log_info(const char* restrict format, ...) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write information to stderr, using a va_list of format-arguments
 *
 * @param format Information format (printf)
 * @param ... Possible printf arguments
 */
void vlog_info(const char* restrict format, va_list va);
/**
 * @brief Write a debug message to stderr
 *
 * @param format debug message format (printf)
 * @param ... Possible printf arguments
 */
void log_debug(const char* restrict format, ...) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write a debug message to stderr, using a va_list of format-arguments
 *
 * @param format debug message format (printf)
 * @param ... Possible printf arguments
 */
void vlog_debug(const char* restrict format, va_list va);
