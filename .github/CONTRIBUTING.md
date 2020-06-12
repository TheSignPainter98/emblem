# Contributing

When contributing to this repository, please first discuss the change you wish to make via issue, email, or any other method with the owners of this repository before making a change.

Please note we have a code of conduct, please follow it in all your interactions with the project.

## Coding Style

We have the following guidelines for coding style in C:

- Functions are written in snake case, e.g. `function_name`
- Declared types are written in pascal caps, e.g. `DocAst`
	- Where a `typedef` is used, the thing which is being aliased should have an indicator appended
	- For an `enum` is being aliased, the original needs `_e` appended, e.g. `typedef enum Something_e {...} Something;`
	- For an `struct` is being aliased, the original needs `_s` appended, e.g. `typedef enum Something_s {...} Something;`
	- For an `union` is being aliased, the original needs `_u` appended, e.g. `typedef enum Something_u {...} Something;`
- Local variables, constants and members (e.g. of a struct or union) should be written in camel caps, e.g. `someLocal`
- Global variables and constants should be written in Pascal caps, e.g. `SomeGlobal`
- Preprocessor definitions should be written in capital snake case, e.g. `SOME_DEF`

## Pull Request Process

When authoring a PR, please use the template provided.

1. Ensure any install or build dependencies are removed before the end of the layer when doing a build.
2. Update the README.md with details of changes to the interface, this includes new environment variables, exposed ports, useful file locations and container parameters.
3. Update `emperor.json` to reflect these changes where applicable.
4. Increase the version numbers in any examples files and the README.md to the new version that this Pull Request would represent. The versioning scheme we use is [SemVer](http://semver.org/).
5. You may merge the Pull Request once you have signed off with two members of the `emperor-lang` team. (Note that for team-members we only require one other member as this is quite a small project at the moment.)
<!-- 5. You may merge the Pull Request in once you have the sign-off of two other developers, or if you do not have permission to do that, you may request the second reviewer to merge it for you. -->
