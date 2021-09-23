# Writing an Extension

It is deliberately quite simple to write and use an extension in Emblem---let's go through the basics.

An extension is a set of functions which may be called from an Emblem document.
For example, the Emblem function to create a level-1 header (like HTML's `<h1>` tag, as seen at the top of this page), behind a little syntactic sugar is just a call to the `.h1` function in the Emblem standard library.

When a function is called, Emblem implicitly searches to find which code to execute, runs it, applies the CSS style using a class of the same name, and places it into the document.
In the example of the `.h1` function, calling may add an entry to the table of contents and obtain the correct section number, but it is the `.h1` class which provides the appearance (such as bold, or with extra vertical spacing).
