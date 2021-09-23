# `.case`

The case directive is used to select one of a number of cases by index.
These cases are 1-indexed, so the first is referenced by index 1.
So for example if `!index` is 2, the following will be evaluated to ‘World.’

```emblem
.case{!index}:
	Hello
::
	World
::
	How are you?
```

If the index supplied is higher than the number of cases present or is negative, the last case is returned as a fallback option, hence if above `!index` is -1 or 12 for example, ‘How are you?’ will be returned.

## Example -- Chinese calendar year name

The following could be used to translate from the year number in the Gregorian calendar to its Chinese calendar name, assuming the current year is stored in `!year`.

```emblem
.case{.add{1}: .mod{!year}: 12}{Rooster}{Dog}{Pig}{Rat}{Ox}{Tiger}{Rabbit}{Dragon}{Snake}{Horse}{Goat}{Monkey}
```
