# djfmt

A formatter for Django templates.

Currently a work in progress, so don't expect it to work just yet.

The goal is to provide a tool that can format Django templates in a consistent way, similar to how `black` formats Python code. In the future it may also provide some linting capabilities.

## Performance

Initial tests suggest `djfmt` is ~150x faster than `djhtml` for formatting files on my Macbook M3 Pro:

```bash
hyperfine --warmup 3 -N 'djlint --quiet --reformat ./tests/django' './target/release/djfmt ./tests/django'

###

Benchmark 1: djlint --quiet --reformat ./tests/django
  Time (mean ± σ):     231.2 ms ±   1.4 ms    [User: 471.0 ms, System: 60.6 ms]
  Range (min … max):   229.1 ms … 234.0 ms    13 runs

Benchmark 2: ./target/release/djfmt ./tests/django
  Time (mean ± σ):       1.5 ms ±   0.3 ms    [User: 0.7 ms, System: 3.1 ms]
  Range (min … max):     1.3 ms …   6.3 ms    1828 runs

Summary
  ./target/release/djfmt ./tests/django ran
  152.68 ± 27.31 times faster than djlint --quiet --reformat ./tests/django
```
