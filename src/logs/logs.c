#include "logs.h"

#include <pthread.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * @brief Verbosity level
 */
typedef enum
{
	/** @brief Error verbosity level */
	ERR,
	/** @brief Warning verbosity level */
	WARN,
	/** @brief Information verbosity level */
	INFO,
	/** @brief Success message verbosity level */
	SUCC,
} Verbosity;

static Verbosity log_verbosity;

/**
 * @brief Log message prefixes with colour
 */
static const char* const leaders[] = {
	[WARN] = "\033[1;33mwarn\033[1;37m:\033[0m ",
	[ERR] = "\033[1;31mfail\033[1;37m:\033[0m ",
	[INFO] = "\033[1;34minfo\033[1;37m:\033[0m ",
	[SUCC] = "\033[1;32msucc\033[1;37m:\033[0m ",
};

static pthread_mutex_t log_lock;

void init_logs(Args* args)
{
	log_verbosity = args->verbose;
	pthread_mutex_init(&log_lock, NULL);
}

void fini_logs(void) { pthread_mutex_destroy(&log_lock); }

static void log_x(Verbosity lvl, const char* restrict format, va_list va);

#define LOG_X_CALL(lvl, v) \
	va_list va;\
	va_start(va, v);\
	log_x(lvl, v, va);\
	va_end(va);

/**
 * @brief Write a warning to stderr
 *
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
void log_warn(const char* restrict format, ...)
{
	LOG_X_CALL(WARN, format)
}

/**
 * @brief Write an error stderr
 *
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void log_err(const char* restrict format, ...)
{
	LOG_X_CALL(ERR, format)
}

/**
 * @brief Write information to stderr
 *
 * @param format Information format (printf)
 * @param ... Possible printf arguments
 */
void log_info(const char* restrict format, ...)
{
	LOG_X_CALL(INFO, format);
}

/**
 * @brief Write a success message to stderr
 *
 * @param format Success message format (printf)
 * @param ... Possible printf arguments
 */
void log_succ(const char* restrict format, ...)
{
	LOG_X_CALL(SUCC, format);
}

static void log_x(Verbosity lvl, const char* restrict format, va_list va)
{
	if (log_verbosity >= lvl)
	{
		pthread_mutex_lock(&log_lock);

		const char* const leader = leaders[lvl];
		size_t leaderLen = strlen(leader);
		va_list va2;
		va_copy(va2, va);
		size_t msgLen = vsnprintf(NULL, 0, format, va2);
		size_t outStrLen = 2 + leaderLen + msgLen;
		char* outStr = malloc(outStrLen * sizeof(char));

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
		strncpy(outStr, leader, leaderLen);
		vsnprintf(outStr + leaderLen, msgLen + 1, format, va);
		outStr[leaderLen + msgLen] = '\n';
		outStr[leaderLen + msgLen + 1] = '\0';

		fprintf(stderr, outStr);

		pthread_mutex_unlock(&log_lock);

		free(outStr);
	}
}
