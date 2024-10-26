# djfmt

A formatter for Django templates.

Currently a work in progress, so don't expect it to work just yet.

The goal is to provide a tool that can format Django templates in a consistent way, similar to how `black` formats Python code. In the future it may also provide some linting capabilities.

## Performance

Initial tests suggest `djfmt` is ~215x faster than `djhtml` for formatting files on my Macbook M3 Pro:

```bash
hyperfine --warmup 3 -N -i 'djlint --quiet --check ./tests/formatter/django/input/2.html' './target/release/djfmt ./tests/formatter/django/input/2.html'

###

Benchmark 1: djlint --quiet --check ./tests/formatter/django/input/2.html
  Time (mean ± σ):     245.4 ms ±   1.5 ms    [User: 191.7 ms, System: 15.9 ms]
  Range (min … max):   243.7 ms … 248.1 ms    12 runs

Benchmark 2: ./target/release/djfmt ./tests/formatter/django/input/2.html
  Time (mean ± σ):       1.1 ms ±   0.1 ms    [User: 0.5 ms, System: 0.3 ms]
  Range (min … max):     1.0 ms …   1.5 ms    2628 runs

Summary
  ./target/release/djfmt ./tests/formatter/django/input/2.html ran
  218.17 ± 12.27 times faster than djlint --quiet --check ./tests/formatter/django/input/2.html
```

```bash
hyperfine --warmup 3 -N -i 'djlint --quiet --check ./tests/formatter/django/input' './target/release/djfmt ./tests/formatter/django/input'

###

Benchmark 1: djlint --quiet --check ./tests/formatter/django/input
  Time (mean ± σ):     319.9 ms ±   1.5 ms    [User: 352.6 ms, System: 37.3 ms]
  Range (min … max):   317.6 ms … 321.7 ms    10 runs

Benchmark 2: ./target/release/djfmt ./tests/formatter/django/input
  Time (mean ± σ):       1.4 ms ±   0.3 ms    [User: 0.8 ms, System: 1.4 ms]
  Range (min … max):     1.2 ms …   6.7 ms    2128 runs

Summary
  ./target/release/djfmt ./tests/formatter/django/input ran
  235.31 ± 43.96 times faster than djlint --quiet --check ./tests/formatter/django/input
```
