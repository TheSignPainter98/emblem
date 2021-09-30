h1s = Array.from(document.getElementsByTagName('h1'));

h1s.forEach((h1) => {
	code = Array.from(h1.getElementsByTagName('code'));
	if (code.length && code[0].textContent.charAt(0) == '.')
		h1.classList.add('directive');
});

root = document.getElementsByTagName('body')[0];
is_sans = false;
toggleSans = () => {
	is_sans = !is_sans;
	if (is_sans)
		root.classList.add('serif');
	else
		root.classList.remove('serif');
};
