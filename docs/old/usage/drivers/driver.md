# Writing a Driver

Emblem is not tied to a single output format, rather, it performs typesetting in a generic fashion, before handing the generated document to a driver.
It is then the job of the driver to translate the positioning of item into the appropriate format.

A driver is a dynamically-loaded C library which contains a number of functions.

1. `const char* get_version(void)`, which returns the version of the driver for diagnostic output
2. `int init_driver(int argc, char** argv)`, which is passed the command-line arguments pertaining to the driver for appropriate parsing
3. `int drive(const char* ifname, EmblemTypesetAst* ast)`, which directs the driver to parse the `ast` into its output format, given that it was generated from a file called `ifname`

Should any functions be required to initialise the library _before_ the `init_driver` function, or at the end of operation, they should have the following minimal prototype.

```c
void something_run_at_loading(void) __attribute__((constructor));
void something_run_at_unloading(void) __attribute__((destructor));
```
