# .eq, .numeq, .streq

In Emblem, there are three modes of equality: structural, textual and numerical.
These correspond to the equality functions `.eq`, `.streq` and `.numeq` respectively.
Although all compute equality, the differences between each should be noted.

## Structural equality -- `.eq`

### Example - Identical structures
### Example - Non-equal, apparently identical structures

## Textual equality -- `.streq`

### Example - Non-identical structures with textual equality 1
```emblem
**something emphasised**
_something emphasised_
```
### Example - Non-identical structures with textual equality 2
```emblem
.it{two words}
.it{two}{words}
```
### Example - Different structures with textual equality

## Numeric equality

### Example - Printing multiples of 2
