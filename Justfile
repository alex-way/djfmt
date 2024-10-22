benchmark:
	hyperfine --warmup 3 -N 'djlint --quiet --reformat ./tests/formatter/django/expected/2.html' './target/release/djfmt ./tests/formatter/django/expected/2.html'
benchmark-dir:
	hyperfine --warmup 3 -N 'djlint --quiet --reformat ./tests/formatter/django/expected' './target/release/djfmt ./tests/formatter/django/expected'
