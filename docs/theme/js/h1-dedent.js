h1s = Array.from(document.getElementsByTagName('h1'));

h1s.forEach((h1,_) => {
	code = Array.from(h1.getElementsByTagName('code'));
	if (code.length && code[0].textContent.charAt(0) == '.')
		h1.classList.add('directive');
});
