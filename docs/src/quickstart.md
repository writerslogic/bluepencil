# Quick start

```sh
cd my-novel
bluepencil init
```

Edit `project.files` in the new `bluepencil.toml` to match your chapter files, then:

```sh
bluepencil report
bluepencil echoes
bluepencil starters chapters/07.md
```

You can always pass files directly, or pipe text in with `-`:

```sh
pbpaste | bluepencil rhythm -
```
