#!/usr/bin/awk -f

BEGIN {
	if (!module_name)
	{
		print "fhdjskal" >"/dev/stderr"
		exit 1
	}
}

{
	if (NR > 1)
		print str
	str = $0
}

END {
	if (substr(str, 0, 1) == "{")
		printf "package.preload['%s'] = -> %s\n", module_name, str
	print str
}
