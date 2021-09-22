# `.gt`, `.ge`, `.lt`, `.le`

The inequality functions take input of numerical values and return a boolean value as one might expect them to in maths.
At their heart, they take pairs of numbers and check whether they satisfy the relevant condition:

- `.gt`---Greater than
- `.ge`---Greater than or equal
- `.lt`---Less than
- `.le`---Less than or equal

## Example -- Checking pairs of numbers

The following are simple examples of inequalities

```emblem
.gt{123}{321} // Evaluates to false
.ge{123}{321} // Evaluates to false
.lt{123}{321} // Evaluates to true
.le{123}{321} // Evaluates to true
.gt{456}{456} // Evaluates to false
.ge{456}{456} // Evaluates to true
.lt{456}{456} // Evaluates to false
.le{456}{456} // Evaluates to true
```

## Example -- Checking more than two numbers

Sometimes, multiple numbers must be checked, and in this case maths often uses the following shorthand:

\\( a \leq b \leq c \Longleftrightarrow a \leq b \mathrel{\\&} b \leq c \\)

The directives we discuss here also allow the same functionality---if more than two arguments are presented, from left to right, each consecutive pair is checked for inequality and a true value is only returned if all these conditions hold.
It should be noted that computation can be halted early, so later arguments will not be evaluated if an earlier pair evaluates to a false value.
