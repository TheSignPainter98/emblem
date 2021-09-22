# `.if`

This directive allows for the selection of a single branch (or none) by use of a condition.
This directive takes two forms:

```emblem
.if{condition}{branch}
.if{condition}{branch 1}{branch 2}
```

The two-argument form operates as follows:

- If _condition_ is true, _branch_ is returned
- Otherwise nothing is returned

The three-argument form operates as follows:[^ifelse]

- If _condition_ is true, _branch 1_ is returned
- Otherwise, _branch 2_ is returned

## Example -- Greetings message

Assuming that the variable `!is_morning` contains a value which represents whether the current time is morning, we could tailor the user’s greeting message to the current time.

```emblem
.echo:
	.if{!is_morning}:
		Good morning!
	::
		Hello!
```

[^ifelse]: This form operates exactly the same as an if-else block common to many programming languages.
