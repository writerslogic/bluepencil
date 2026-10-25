# Continuous integration

## GitHub Actions

To show bluepencil observations at their source locations in a pull request workflow:

```yaml
name: Prose review
on: pull_request
permissions:
  contents: read
jobs:
  prose:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: writerslogic/bluepencil/.github/actions/bluepencil@main
        with:
          base-sha: ${{ github.event.pull_request.base.sha }}
```

The action reviews changed Markdown, Fountain, and plain-text files, adds up to 25 location-aware notices, and writes a workflow summary. It builds bluepencil from the action's source, so the first run takes longer. For a stable production workflow, pin the action to a release commit after publication.

For threshold-based pass/fail gating, configure `[check]` and run:

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
