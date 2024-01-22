# freqle

Extremely simple CLI tool for maintaining a [frecency](https://en.wikipedia.org/wiki/Frecency) history.
This is a Rust port of [frecently](https://github.com/jonascarpay/frecently).

The intended use case is to add a frecency-based search history to CLI tools like [dmenu](https://tools.suckless.org/dmenu/), [rofi](https://github.com/davatorium/rofi), or [fzf](https://github.com/junegunn/fzf).
It can also emulate directory-jumping tools, like [autojump](https://github.com/wting/autojump) or [z](https://github.com/rupa/z).


### Examples

These are some examples of how to integrate `freqle` with other tools.
For more detailed information, run `freqle --help`.

#### Basic CLI:

```console
$ freqle view .history      # .history doesn't exist yet. By default, all commands treat a missing file as empty
$ freqle bump .history foo  # creates .history, and bumps foo
$ freqle bump .history bar
$ freqle view .history      # `view` shows all entries, ordered by frecency
bar
foo
$ echo -e "bar\nbaz" | freqle view .history --augment   # with --augment, freqle accepts extra entries on stdin (newline-separated) that should always appear in the output
bar
foo
baz
$ echo -e "foo\nbar\nbaz" | freqle view .history --augment --scores   # --scores allows us to inspect the internal state
weighted score  hourly          daily           monthly
  178.663539      0.985473        0.999390        0.999980      bar
  178.267277      0.983010        0.999286        0.999976      foo
    0.000000      0.000000        0.000000        0.000000      baz
$ echo -e "bar\nbaz" | freqle view .history --augment --restrict  # with --restrict, we exclusively output entries that appear on stdin
bar
baz
```

#### `dmenu` web searches with history

This bash script shows an example of how `freqle` integrates with tools like `dmenu`:

```bash
set -eo pipefail
HISTORY=~/.search-history
QUERY=$(freqle view $HISTORY | dmenu -p "Web search:")

if [[ -n "$QUERY" ]]; then
  freqle bump "$HISTORY" "$QUERY"
  xdg-open "https://www.duckduckgo.com/?q=$QUERY"
fi
```

#### Directory picker

In this more complicated bash script, we use `freqle` and `dmenu` to first ask the user for a directory, and then open a terminal in that directory.
It shows a good use-case for `--augment` (`-a`), augmenting the list with any non-hidden directory at most 2 deep from `$HOME`.

```bash
set -eo pipefail
HISTORY=~/.directory-history
# First, purge non-existent directories from the history
for dir in $(freqle view $HISTORY); do
  if [ ! -d "$dir" ]; then
    echo "Removing $dir"
    freqle delete $HISTORY "$dir"
  fi
done
# Load the history, augmenting it with every non-hidden directory at most 2 deep from $HOME.
DIR=$(find $HOME -maxdepth 2 -type d -not -path '*/.*' | freqle view $HISTORY -a | dmenu -i)
if [ -d $DIR ]; then
  freqle bump $HISTORY "$DIR"
  $TERMCMD -d "$DIR"
fi
```

This kind of script is especially useful when combined with a shell hook that bumps on every directory change, to get `autojump`-like behavior.
For `fish`, that looks like this:

```fish
function __frecently-directory-hook --on-variable PWD --description 'bump current directory in history'
  freqle bump /path/to/history/file "$PWD"
end
```

### Installation

#### Binaries

The [GitHub Action CI](https://github.com/jonascarpay/freqle/actions) builds static binaries for x86 and aarch64, so look for artifacts on the most recent action.
If this project gains traction (stars) I'll add proper releases so the artifacts don't get deleted after 90 days.

`freqle` is also available on [Hackage](http://hackage.haskell.org/package/frecently), so depending on your distro you might be able to install straight from there.

#### Compiling from source

`frecently` can be built using a Haskell build tool, or using Nix.

Using Cabal, run `cabal build`.

Using Nix, the `flake.nix` file exposes the executable both directly and as an overlay.

### Implementation details

`frecently` works by maintaining three energy levels per entry.
The energy levels decay exponentially, with half-lives of an hour, a day, and a month, respectively.
We `bump` an entry by adding 1 to each of these.

An entry's frecency score is calculated by multiplying each of these three energies by a weight.
The weights default to 720, 30, and 1, for the hourly, daily, and monthly energies, respectively, but can be overridden on the CLI.

Energies are updated _only_ when the history file is used in a `bump` command, and when we do, we update every entry's energy simultaneously.
This is invisible to the user, but it ensures that we only need to calculate decay factors once when opening a file, making score calculations very efficient.

When, during an update, an entry's monthly energy drops below the threshold value (defaults to 0.1), it is deleted from the history.
If you don't want items to be deleted, use a threshold of 0.

### Comparison with other tools

This tool was inspired by [frece](https://github.com/YodaEmbedding/frece).
I really like the idea behind `frece`, but I think the execution is more complicated than it needs to be.
`freqle` is both simpler and easier to integrate into CLI applications.
