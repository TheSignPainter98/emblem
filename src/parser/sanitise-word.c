#include "sanitise-word.h"

#include "logs/logs.h"
#include <limits.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#define ONE_CHAR_MASK ((1 << CHAR_BIT) - 1)

typedef struct
{
	enum
	{
		NO_MARK,
		ESCAPE,
		REMOVED,
		SUBSTITUTION
	} type;
	size_t sub_len;
	char* substitution;
} Mark;

typedef struct
{
	Mark* marks;
	size_t out_len;
	bool seen_non_quote_mark;
} SanitiserState;

typedef struct
{
	size_t replacement_len;
	char* replacement;
} SingleSubstitution;

typedef struct
{
	size_t left_replacement_len;
	char* left_replacement;
	size_t right_replacement_len;
	char* right_replacement;
} PairSubstitution;

typedef struct
{
	size_t needle_len;
	char* needle;
	enum
	{
		SINGLE_SUBSTITUTION,
		PAIR_SUBSTITUTION,
	} substitution_type;
	union
	{
		SingleSubstitution single;
		PairSubstitution pair;
	};
} Substitution;

static Substitution subs[] = {
	{ 0, "---", SINGLE_SUBSTITUTION, .single = { 0, "—" } },
	{ 0, "--", SINGLE_SUBSTITUTION, .single = { 0, "–" } },
	{ 0, "...", SINGLE_SUBSTITUTION, .single = { 0, "…" } },
	{ 0, "<=", SINGLE_SUBSTITUTION, .single = { 0, "≤" } },
	{ 0, ">=", SINGLE_SUBSTITUTION, .single = { 0, "≥" } },
	{ 0, "'", PAIR_SUBSTITUTION, .pair = { 0, "‘", 0, "’" } },
	{ 0, "\"", PAIR_SUBSTITUTION, .pair = { 0, "“", 0, "”" } },
};

static const char valid_escape_chars[] = {
	'{',
	'}',
	'\\',
	':',
	'-',
	'_',
	'*',
	'`',
	'=',
	'\'',
	'"',
	'.',
	',',
	'!',
	'[',
	'@',
	'#',
	'<',
	'>',
};

static bool is_valid_escape_char(char c);
static void compute_mark(SanitiserState* state, size_t word_len, char* word, size_t* pos);
static bool matches_needle(Substitution* sub, size_t word_len, char const* word, size_t pos);

static void init_word_sanitiser(void) __attribute__((constructor));
static void init_word_sanitiser(void)
{
	for (size_t i = 0; i < sizeof(subs) / sizeof(*subs); i++)
	{
		subs[i].needle_len = strlen(subs[i].needle);
		switch (subs[i].substitution_type)
		{
			case SINGLE_SUBSTITUTION:
				subs[i].single.replacement_len = strlen(subs[i].single.replacement);
				break;
			case PAIR_SUBSTITUTION:
				subs[i].pair.left_replacement_len  = strlen(subs[i].pair.left_replacement);
				subs[i].pair.right_replacement_len = strlen(subs[i].pair.right_replacement);
				break;
			default:
				fprintf(stderr, "Substitution has unknown type: %d\n", subs[i].substitution_type);
				exit(1);
		}
	}
}

char* sanitise_word(EM_LTYPE* yylloc, Str* ifn, char* word, size_t len)
{
	Mark marks[len];
	SanitiserState state = {
		.marks				 = marks,
		.out_len			 = len,
		.seen_non_quote_mark = false,
	};

	for (size_t i = 0; i < len; i++)
		compute_mark(&state, len, word, &i);

	char* new_word	= calloc(1 + state.out_len, sizeof(char));
	char* new_wordp = new_word;

	for (size_t i = 0; i < len; i++)
		switch (state.marks[i].type)
		{
			case NO_MARK:
				*(new_wordp++) = word[i];
				break;
			case REMOVED:
				break;
			case ESCAPE:
				*(new_wordp++) = word[++i];

				// Warn of unknown escape characters
				if (!is_valid_escape_char(word[i]))
				{
					Location eloc = {
						.first_line	  = yylloc->first_line,
						.first_column = yylloc->first_column + i,
						.last_line	  = yylloc->last_line,
						.last_column  = yylloc->first_column + i + 1,
						.src_file	  = ifn,
					};

					if (log_warn_at(&eloc, "Unrecognised character escape '\\%c' (%#02x)", word[i] ? word[i] : '0',
							word[i] & ONE_CHAR_MASK))
						exit(1);
				}
				break;
			case SUBSTITUTION:
				memcpy(new_wordp, state.marks[i].substitution, state.marks[i].sub_len);
				new_wordp += state.marks[i].sub_len;
				break;
			default:
				log_err("Unknown mark %d generated at index %ld in word {%s} (This is a bug, you shouldn't be seeing "
						"this!)",
					state.marks[i].type, new_wordp - new_word, word);
				exit(1);
		}
	*new_wordp = '\0';

	return new_word;
}

static void compute_mark(SanitiserState* state, size_t word_len, char* word, size_t* pos)
{
	for (size_t i = 0; i < sizeof(subs) / sizeof(*subs); i++)
	{
		if (word[*pos] == '\\')
		{
			state->marks[*pos].type	   = ESCAPE;
			state->marks[++*pos].type  = NO_MARK;
			state->seen_non_quote_mark = true;
		}
		else if (!matches_needle(&subs[i], word_len, word, *pos))
			continue;
		else
		{
			switch (subs[i].substitution_type)
			{
				case SINGLE_SUBSTITUTION:
					state->marks[*pos].substitution = subs[i].single.replacement;
					state->marks[*pos].sub_len		= subs[i].single.replacement_len;
					state->out_len += state->marks[*pos].sub_len;
					break;
				case PAIR_SUBSTITUTION:
					if (state->seen_non_quote_mark)
					{
						state->marks[*pos].substitution = subs[i].pair.right_replacement;
						state->marks[*pos].sub_len		= subs[i].pair.right_replacement_len;
					}
					else
					{
						state->marks[*pos].substitution = subs[i].pair.left_replacement;
						state->marks[*pos].sub_len		= subs[i].pair.left_replacement_len;
					}
					state->out_len += state->marks[*pos].sub_len;
					break;
				default:
					log_err("Unknown substitution type: %d (This is a bug, you shouldn't be seeing this!)",
						subs[i].substitution_type);
					exit(1);
			}

			// Handle marks
			state->marks[*pos].type = SUBSTITUTION;
			for (size_t j = 1; j < subs[i].needle_len; j++)
				state->marks[*pos + j].type = REMOVED;

			// Remove needle from output length
			state->out_len -= subs[i].needle_len;

			// Update the inspection position
			*pos += subs[i].needle_len - 1;
		}
		return;
	}
	state->marks[*pos].type	   = NO_MARK;
	state->seen_non_quote_mark = true;
}

static bool matches_needle(Substitution* sub, size_t word_len, char const* word, size_t pos)
{
	if (word_len < pos + sub->needle_len)
		return false;

	for (size_t j = 0; j < sub->needle_len; j++)
	{
		if (word[pos + j] != sub->needle[j])
			return false;
	}
	return true;
}

static bool is_valid_escape_char(char c)
{
	for (size_t i = 0; i < sizeof(valid_escape_chars) / sizeof(*valid_escape_chars); i++)
		if (c == valid_escape_chars[i])
			return true;
	return false;
}
