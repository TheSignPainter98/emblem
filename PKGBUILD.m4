# Maintainer: Ed Jones <emdev@kcza.net>
# Contributor: Ed Jones <emdev@kcza.net>

pkgname=S_PKGNAME
pkgver=S_PKGVER
pkgrel=1
source=('S_SRC')
arch=('x86_64')
license=('GPL3')
pkgdesc='S_PKGDESC'
url=https://github.com/TheSignPainter98/emblem
changelog=ChangeLog
depends=(S_DEPENDS)
makedepends=(S_MAKEDEPENDS)
checkdepends=(S_CHECKDEPENDS)
optdepends=(S_CHECKDEPENDS)
sha256sums=('S_SHA256SUM')
sha512sums=('S_SHA512SUM')

build()
{
	cd "$srcdir/$pkgname-$pkgver"
	./configure -q --prefix=/usr
	make -s CFLAGS=-Wno-error
}

package()
{
	cd "$srcdir/$pkgname-$pkgver"
	make -s DESTDIR="$pkgdir/" install-am
}

check()
{
	cd "$srcdir/$pkgname-$pkgver"
	make -s check_em CFLAGS=-Wno-error
	./check_em
}
