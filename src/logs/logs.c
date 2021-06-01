#include "logs.h"

#include <pthread.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define POS_FMT		   "%s:%d:%d: "
#define POS_FILL(locp) locp->src_file->str, (locp)->first_line, (locp)->first_column
#define PREPEND_LOC_TO_FORMAT(posstr, loc)                                                                             \
	const size_t pos_fmt_len = 1 + snprintf(NULL, 0, POS_FMT, POS_FILL(loc));                                          \
	char(posstr)[pos_fmt_len];                                                                                         \
	snprintf(posstr, pos_fmt_len, POS_FMT, POS_FILL(loc));

/**
 * @brief Verbosity level values
 */
typedef enum
{
	/** @brief Error verbosity level */
	VERBOSITY_ERR,
	/** @brief Warning verbosity level */
	VERBOSITY_WARN,
	/** @brief Information verbosity level */
	VERBOSITY_INFO,
	/** @brief Debug message verbosity level */
	VERBOSITY_DEBUG,
} Verbosity;

/**
 * @brief Running verbosity level of the program
 */
static Verbosity log_verbosity;

static bool fatal_warnings;

static bool colourise_output;

/**
 * @brief Log message prefixes with colour
 */
static const char* const leaders[2][4] = {
	[true] = {
		[VERBOSITY_WARN] = "\033[1;33mwarning\033[1;37m:\033[0m ",
		[VERBOSITY_ERR] = "\033[1;31merror\033[1;37m:\033[0m ",
		[VERBOSITY_INFO] = "\033[1;32minfo\033[1;37m:\033[0m ",
		[VERBOSITY_DEBUG] = "\033[1;34mdebug\033[1;37m:\033[0m ",
	},
	[false] = {
		[VERBOSITY_WARN] = "warning: ",
		[VERBOSITY_ERR] = "error: ",
		[VERBOSITY_INFO] = "info: ",
		[VERBOSITY_DEBUG] = "debug: ",
	},
};

/**
 * @brief Lock to prevent multiple threads from simultaneously logging
 */
static pthread_mutex_t log_lock;

#include <curses.h>
#include <errno.h>
void init_logs(Args* args)
{
	log_verbosity  = args->verbose;
	fatal_warnings = args->fatal_warnings;
	if (args->colourise_output)
		colourise_output = args->colourise_output;
	else
	{
		initscr();
		colourise_output = isatty(2) && has_colors();
		endwin();
	}
	pthread_mutex_init(&log_lock, NULL);
}

void fini_logs(void) { pthread_mutex_destroy(&log_lock); }

static void log_x(Verbosity lvl, const char* prefix, const char* restrict format, va_list va);

/**
 * @brief Construct a call to the logging function at verbosity `lvl`, where `v` is the start of the formatting
 * arguments
 *
 * @param lvl Verbosity level of the call
 * @param v Name of the first format argument
 *
 * @return A call to log_x with va_args handled
 */
#define LOG_X_CALL(name, f)                                                                                            \
	va_list va;                                                                                                        \
	va_start(va, f);                                                                                                   \
	vlog_##name(f, va);                                                                                                \
	va_end(va)

/**
 * @brief Write a warning to stderr
 *
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
int log_warn(const char* restrict format, ...)
{
	LOG_X_CALL(warn, format);
	return fatal_warnings;
}

int vlog_warn(const char* restrict format, va_list va)
{
	if (fatal_warnings)
		log_x(VERBOSITY_ERR, "", format, va);
	else
		log_x(VERBOSITY_WARN, "", format, va);
	return fatal_warnings;
}

int log_warn_at(Location* loc, const char* restrict format, ...)
{
	va_list va;
	va_start(va, format);
	vlog_warn_at(loc, format, va);
	va_end(va);
	return fatal_warnings;
}

int vlog_warn_at(Location* loc, const char* restrict format, va_list va)
{
	PREPEND_LOC_TO_FORMAT(locstr, loc);
	log_x(VERBOSITY_WARN, locstr, format, va);
	return fatal_warnings;
}

/**
 * @brief Write an error stderr
 *
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void log_err(const char* restrict format, ...) { LOG_X_CALL(err, format); }

void vlog_err(const char* restrict format, va_list va) { log_x(VERBOSITY_ERR, "", format, va); }

void log_err_at(Location* loc, const char* restrict format, ...)
{
	va_list va;
	va_start(va, format);
	vlog_err_at(loc, format, va);
	va_end(va);
}

void vlog_err_at(Location* loc, const char* restrict format, va_list va)
{
	PREPEND_LOC_TO_FORMAT(locstr, loc);
	log_x(VERBOSITY_ERR, locstr, format, va);
}

/**
 * @brief Write information to stderr
 *
 * @param format Information format (printf)
 * @param ... Possible printf arguments
 */
void log_info(const char* restrict format, ...) { LOG_X_CALL(info, format); }

void vlog_info(const char* restrict format, va_list va) { log_x(VERBOSITY_INFO, "", format, va); }

/**
 * @brief Write a debug message to stderr
 *
 * @param format debug message format (printf)
 * @param ... Possible printf arguments
 */
void log_debug(const char* restrict format, ...) { LOG_X_CALL(debug, format); }

void vlog_debug(const char* restrict format, va_list va) { log_x(VERBOSITY_DEBUG, "", format, va); }

static void log_x(Verbosity lvl, const char* prefix, const char* restrict format, va_list va)
{
	if (log_verbosity >= lvl)
	{
		pthread_mutex_lock(&log_lock);

		size_t prefix_len = strlen(prefix);

		const char* const leader = leaders[colourise_output][lvl];
		size_t leaderLen		 = strlen(leader);
		va_list va2;
		va_copy(va2, va);
		size_t msgLen	 = vsnprintf(NULL, 0, format, va2); // NOLINT
		size_t outStrLen = 4 + prefix_len + leaderLen + msgLen;
		char* outStr	 = malloc(outStrLen * sizeof(char));

		// Handle if out of memory
		if (!outStr)
		{
			fprintf(stderr, "Failed to allocate space for formatted string during output when outputting:\n");
			vfprintf(stderr, format, va);
			fprintf(stderr, "Exiting...\n");
			pthread_mutex_unlock(&log_lock);
			exit(1);
		}

		// Format output message
		char* outStrp = outStr;
		strncpy(outStrp, prefix, prefix_len + 1);
		outStrp += prefix_len;
		strncpy(outStrp, leader, leaderLen);
		outStrp += leaderLen;
		vsnprintf(outStrp, msgLen + 1, format, va);
		outStrp += msgLen;
		*++outStrp = '\n';
		*++outStrp = '\0';

		/* outStr[leaderLen + msgLen]	   = '\n'; */
		/* outStr[leaderLen + msgLen + 1] = '\0'; */

		fputs(outStr, stderr);
		fputs("\n", stderr);

		pthread_mutex_unlock(&log_lock);

		free(outStr);
		va_end(va2);
	}
}
