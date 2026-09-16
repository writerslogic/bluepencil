# Continuous integration

## GitHub Actions

```yaml
- run: cargo install bluepencil
- run: bluepencil check
```

## pre-commit hook

```sh
#!/bin/sh
files=$(git diff --cached --name-only --diff-filter=ACM -- '*.md' '*.fountain')
[ -z "$files" ] || bluepencil check $files
```
