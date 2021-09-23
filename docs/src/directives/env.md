# `.env`

The `.env` directive provides a way to read external environment variables.
It takes input of an environment variable name and returns its value.
If the environment variable does not exist, the empty string is returned.

## Example -- Getting the name of the user’s editor

On Linux, it is common to record the name of the user’s text editor in `EDITOR` environment variable.
This can be accessed within an Emblem document as follows.

```emblem
.env: EDITOR // May or may not return ‘vim’
```
