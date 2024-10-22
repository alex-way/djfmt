benchmark:
	hyperfine --warmup 3 -N 'djlint --quiet --reformat ./djangotemplate.html' './target/release/djfmt ./djangotemplate.html'
benchmark-dir:
	hyperfine --warmup 3 -N 'djlint --quiet --reformat ./tests/django' './target/release/djfmt ./tests/django'
