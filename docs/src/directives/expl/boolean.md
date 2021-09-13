# Boolean

Sometimes, it is necessary to combine several conditions into one, more complicated one.

It must be noted that each of these directives consider their input values in the same way as the [flow-control directives][flow-control]:

1. If the value is empty, it is false
2. If the value is ‘0’, it is false
3. If the value is ‘false’ (case-insensitive), it is false
4. Any other value is true

In the following section, we describe an adequate set of operators with which any boolean expression may be represented:

- [`.all`][all]---conjunction, similar to the ‘and’ operator of many programming languages
- [`.any`][any]---disjunction, similar to the ‘or’ operator of many programming languages
- [`.impl`][impl]---implication
- [`.not`][not]---negation
- [`.xor`][xor]---exclusive disjunction

[flow-control]:  ./flow-control.md
[all]: ../all.md
[any]: ../any.md
[impl]: ../impl.md
[not]: ../not.md
[xor]: ../xor.md
