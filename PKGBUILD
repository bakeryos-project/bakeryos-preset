# Maintainer: smtdfc <me.smtdfc@gmail.com>
pkgname=bakeryos-preset
pkgver=1.0.0
pkgrel=1
pkgdesc="Preset tool for BakeryOS"
arch=('x86_64')
url="https://gitlab.com/bakeryos/bakeryos-preset"
license=('GPL-3.0-or-later')
depends=('gcc-libs' 'glibc' 'pacman')
makedepends=('cargo' 'git')
source=()
sha256sums=()

build(){
   cd $startdir
   cargo build --release
}

package() {
   install -Dm755 "$startdir/target/release/bakeryos-preset" "$pkgdir/usr/bin/preset"
   install -Dm644 "$startdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}