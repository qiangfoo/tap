# work

Interactive git worktree manager.

## Install

```sh
brew install qiangfoo/work/work
```

Then add to your `~/.zshrc` or `~/.bashrc`:

```sh
eval "$(work init)"
```

## Usage

```sh
work          # select and switch to a worktree
work add      # create a new worktree
work remove   # remove a worktree
```

## Configuration

Create `~/.config/work.toml` to set a branch prefix:

```toml
branch_prefix = "qiang"
```

This prefixes new branches: `work add` with name `feature` creates branch `qiang-feature`.
