default:
    just --list

build-release:
	cargo build --release

benchmark:
	just build-release
	hyperfine --warmup 3 -N -i 'djlint --quiet --check ./tests/formatter/django/input/2.html' './target/release/djfmt ./tests/formatter/django/input/2.html'

benchmark-dir:
	just build-release
	hyperfine --warmup 3 -N -i 'djlint --quiet --check ./tests/formatter/django/input' './target/release/djfmt ./tests/formatter/django/input'
