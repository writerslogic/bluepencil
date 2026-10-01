# config

Show the effective configuration: built-in defaults merged with the `bluepencil.toml` in use, and which file that was.

```sh
bluepencil config
bluepencil config --json
```

```text
# effective configuration from /home/me/novel/bluepencil.toml
[project]
files = ["chapters/*.md"]

[echoes]
window = 50
...
```

The output is valid TOML, so it doubles as a fully expanded starting point: redirect it to a file and edit from there. With `--json`, the result is an object with `path` (`null` when no file was found) and `config`.
