# Emblem

An elegant, extensible typesetter with a friendly interface and good output.

## Usage

To compile a document, `hello.em` into a pdf, simply run

```sh
em hello
```

The `.em` file extension is optional.
More information is available in the manual and user guide.

## Example Document

An Emblem document might look something a little like this:

```markdown
# Hello, world!

This is a quite _simple_ document with **bold** and other `markdown` syntax.
.centre:
	This is centered
```

## Building

To build the project from scratch, run the following commands:

```sh
./scripts/autogen
./configure
make
```

Once this has worked, you should how be able to run `./em`.
From here, `make install` can be used to install the project.

The test suite can be invoked with `make check`.

## License

This project is licensed under the GNU General Public License version 3.
