// Handle directive protrusion vs general code protrusion
let h1s = Array.from(document.getElementsByTagName('h1'));
h1s.forEach((h1) => {
	let code = Array.from(h1.getElementsByTagName('code'));
	if (code.length && code[0].textContent.charAt(0) != '.')
		h1.classList.add('not-directive');
});

const root = document.getElementsByTagName('body')[0];
const setSerif = (is_serif) => {
	if (is_serif)
		root.classList.add('serif');
	else
		root.classList.remove('serif');
};

const toggleSans = () => {
	is_serif = !is_serif;
	setSerif(is_serif);
	window.localStorage.setItem('is_serif', is_serif.toString());
};

let is_serif = window.localStorage.getItem('is_serif') === 'true';
setSerif(is_serif);
