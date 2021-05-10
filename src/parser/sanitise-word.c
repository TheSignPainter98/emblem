#include "sanitise-word.h"

#include "logs/logs.h"
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

typedef enum
{
	NO_MARK = 0,
	ESCAPE,
	OPEN_SINGLE_QUOTE,
	CLOSE_SINGLE_QUOTE,
	OPEN_DOUBLE_QUOTE,
	CLOSE_DOUBLE_QUOTE,
	NUM_MARKS,
} Mark;

static bool is_valid_escape_char(char c);

static const char* mark_defs[] = {
	[NO_MARK]			 = "",
	[ESCAPE]			 = "",
	[OPEN_SINGLE_QUOTE]	 = "‘",
	[CLOSE_SINGLE_QUOTE] = "’",
	[OPEN_DOUBLE_QUOTE]	 = "“",
	[CLOSE_DOUBLE_QUOTE] = "”",
};

static size_t mark_def_lens[NUM_MARKS];

static const char valid_escape_chars[] = { '{', '}', '\'', '"', '\\', ':', };

static void init_word_sanitiser(void) __attribute__((constructor));
static void init_word_sanitiser(void)
{
	for (size_t i = 0; i < NUM_MARKS; i++)
		mark_def_lens[i] = strlen(mark_defs[i]);
	mark_def_lens[ESCAPE] = -1;
}

char* sanitise_word(char* word, size_t len)
{
	// Resolve quote marks
	Mark marks[len];
	bool seen_non_quote_mark = false;
	for (size_t i = 0; i < len; i++)
	{
		if (word[i] == '\'')
			marks[i] = seen_non_quote_mark ? CLOSE_SINGLE_QUOTE : OPEN_SINGLE_QUOTE;
		else if (word[i] == '"')
			marks[i] = seen_non_quote_mark ? CLOSE_DOUBLE_QUOTE : OPEN_DOUBLE_QUOTE;
		else if (word[i] == '\\')
		{
			marks[i++] = ESCAPE;
			marks[i] = NO_MARK;
			seen_non_quote_mark = true;
		}
		else
		{
			marks[i] = NO_MARK;
			seen_non_quote_mark = true;
		}
	}

	size_t new_len = len;
	for (size_t i = 0; i < len; i++)
		new_len += mark_def_lens[marks[i]];

	char* new_word	= calloc(1 + new_len, sizeof(char));
	char* new_wordp = new_word;

	for (size_t i = 0; i < len; i++)
		if (!marks[i])
			*(new_wordp++) = word[i];
		else if (marks[i] == ESCAPE)
		{
			*(new_wordp++) = word[++i];

			// Warn of unknown escape characters
			if (!is_valid_escape_char(word[i]))
			{
				log_err("Unrecognised character escape '\\%c' (%#02x)", word[i] ? word[i] : '0', word[i] & 0xff);
				exit(1);
			}
		}
		else
		{
			memcpy(new_wordp, mark_defs[marks[i]], mark_def_lens[marks[i]]);
			new_wordp += mark_def_lens[marks[i]];
		}
	*new_wordp = '\0';

	return new_word;
}

static bool is_valid_escape_char(char c)
{
	for (size_t i = 0; i < sizeof(valid_escape_chars) / sizeof(*valid_escape_chars); i++)
		if (c == valid_escape_chars[i])
			return true;
	return false;
}
