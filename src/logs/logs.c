#include "logs.h"

#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * @brief Verbosity level
 */
typedef enum Verbosity_e
{
	/** @brief Error verbosity level */
	ERR,
	/** @brief Warning verbosity level */
	WARN,
	/** @brief Success message verbosity level */
	SUCC,
	/** @brief Information verbosity level */
	INFO,
} Verbosity;

static int log_verbosity;

/**
 * @brief ANSI colourisation symbols for warnings
 */
const char* warningLeader = "\033[1;33mwarning\033[1;37m:\033[0m ";
/**
 * @brief ANSI colourisation symbols for errors
 */
const char* errorLeader	  = "\033[1;31merror\033[1;37m:\033[0m ";
/**
 * @brief ANSI colourisation symbols for information
 */
const char* infoLeader	  = "\033[1;34minfo\033[1;37m:\033[0m ";
/**
 * @brief ANSI colourisation symbols for success messages
 */
const char* succLeader	  = "\033[1;32msuccess\033[1;37m:\033[0m ";

void init_logs(Args* args)
{
	log_verbosity = args->verbose;
}

void fini_logs(void) {}

/**
 * @brief Write a warning to stderr
 *
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
void log_warn(const char* restrict format, ...)
{
	if (log_verbosity >= WARN)
	{
		const size_t warningLeaderLen = strlen(warningLeader);
		va_list va;
		va_start(va, format);
		size_t formatLen	   = strlen(format);
		char* colourisedFormat = malloc((formatLen + warningLeaderLen + 2) * sizeof(char));
		if (colourisedFormat == NULL)
		{
			fprintf(stderr, "Failed to allocate space for formatted string during output when outputting:\n");
			vfprintf(stderr, format, va);
			fprintf(stderr, "Exiting...\n");
			exit(1);
		}

		strcpy(colourisedFormat, warningLeader);
		strcpy(&colourisedFormat[warningLeaderLen], format);
		colourisedFormat[formatLen + warningLeaderLen]	   = '\n';
		colourisedFormat[formatLen + warningLeaderLen + 1] = '\0';

		vfprintf(stderr, colourisedFormat, va);

		free(colourisedFormat);
		va_end(va);
	}
}

/**
 * @brief Write an error stderr
 *
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
 void log_err(const char* restrict format, ...)
{
	if (log_verbosity >= ERR)
	{
		const size_t errorLeaderLen = strlen(errorLeader);
		va_list va;
		va_start(va, format);
		size_t formatLen	   = strlen(format);
		char* colourisedFormat = malloc((formatLen + errorLeaderLen + 2) * sizeof(char));
		if (colourisedFormat == NULL)
		{
			fprintf(stderr, "Failed to allocate space for formatted string during output when outputting:\n");
			vfprintf(stderr, format, va);
			fprintf(stderr, "Exiting...\n");
			exit(1);
		}

		strcpy(colourisedFormat, errorLeader);
		strcpy(&colourisedFormat[errorLeaderLen], format);
		colourisedFormat[formatLen + errorLeaderLen]	 = '\n';
		colourisedFormat[formatLen + errorLeaderLen + 1] = '\0';

		vfprintf(stderr, colourisedFormat, va);

		free(colourisedFormat);
		va_end(va);
	}
}

/**
 * @brief Write information to stderr
 *
 * @param format Information format (printf)
 * @param ... Possible printf arguments
 */
void log_info(const char* restrict format, ...)
{
	if (log_verbosity >= INFO)
	{
		const size_t infoLeaderLen = strlen(infoLeader);
		va_list va;
		va_start(va, format);
		size_t formatLen	   = strlen(format);
		char* colourisedFormat = malloc((formatLen + infoLeaderLen + 2) * sizeof(char));
		if (colourisedFormat == NULL)
		{
			fprintf(stderr, "Failed to allocate space for formatted string during output when outputting:\n");
			vfprintf(stderr, format, va);
			fprintf(stderr, "Exiting...\n");
			exit(1);
		}

		strcpy(colourisedFormat, infoLeader);
		strcpy(&colourisedFormat[infoLeaderLen], format);
		colourisedFormat[formatLen + infoLeaderLen]		= '\n';
		colourisedFormat[formatLen + infoLeaderLen + 1] = '\0';

		vfprintf(stderr, colourisedFormat, va);

		free(colourisedFormat);
		va_end(va);
	}
}

/**
 * @brief Write a success message to stderr
 *
 * @param format Success message format (printf)
 * @param ... Possible printf arguments
 */
void log_succ(const char* restrict format, ...)
{
	if (log_verbosity >= SUCC)
	{
		const size_t succLeaderLen = strlen(succLeader);
		va_list va;
		va_start(va, format);
		size_t formatLen	   = strlen(format);
		char* colourisedFormat = malloc((formatLen + succLeaderLen + 2) * sizeof(char));
		if (colourisedFormat == NULL)
		{
			fprintf(stderr, "Failed to allocate space for formatted string during output when outputting:\n");
			vfprintf(stderr, format, va);
			fprintf(stderr, "Exiting...\n");
			exit(1);
		}

		strcpy(colourisedFormat, succLeader);
		strcpy(&colourisedFormat[succLeaderLen], format);
		colourisedFormat[formatLen + succLeaderLen]		= '\n';
		colourisedFormat[formatLen + succLeaderLen + 1] = '\0';

		vfprintf(stderr, colourisedFormat, va);

		free(colourisedFormat);
		va_end(va);
	}
}
