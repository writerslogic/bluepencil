# Installation

## Homebrew

```sh
brew install writerslogic/tap/bluepencil
```

## Cargo

```sh
cargo install bluepencil
```

## From source

```sh
git clone https://github.com/writerslogic/bluepencil
cd bluepencil
cargo install --path crates/bluepencil
```

## Shell completions

```sh
bluepencil completions zsh > "${fpath[1]}/_bluepencil"
bluepencil completions bash > ~/.local/share/bash-completion/completions/bluepencil
bluepencil completions fish > ~/.config/fish/completions/bluepencil.fish
```
