#!/usr/bin/awk -f

BEGIN {
	if (!module_name)
	{
		print "Specify a module name!" >"/dev/stderr"
		exit 1
	}
}

/^((base|__em)\.)?stylesheet/ && !/,/ {
	name = $2
	gsub("['\"]", "", name)

	stylesheet_loc = "src/ext/lib/" name

	stylesheet_content
	while ((getline line < stylesheet_loc) > 0)
		stylesheet_content = stylesheet_content "\n" line
	if (stylesheet_content)
	{
		if (NR > 1)
			print str
		str = sprintf("__em.stylesheet '%s', '/* Internal stylesheet for module %s */%s'", stylesheet_loc, escape(module_name), escape(stylesheet_content))
	}
	else
		print $0
	next
}

/^import .* from _G$/ {
	next
}

{
	if (NR > 1)
		print str
	str = $0
}

END {
	if (substr(str, 0, 1) == "{" || (str !~ /^--/ && str ~ /^[^ \t{}]*$/))
		printf "package.preload['%s'] = -> %s\n", module_name, str
	print str
}

function escape(str)
{
	gsub("'", "\\'", str)
	return str
}
