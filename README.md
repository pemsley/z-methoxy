
**`z-methoxy`** is a `cd` for `tcsh` that remembers - it works quite like `zoxide`.

For many years, off and on, I'd been trying to get `autojump` to work
with `tcsh`.  I never managed it.  `autojump` was useful (for others)
because it remembered the directories that you had visited, so that,
at a later date you could use that information to visit the directory
again using less memory and key strokes.

Recently I heard a review of [zoxide](https://github.com/ajeetdsouza/zoxide)
on "Linux Dev Time" (or one of the "Linux After Dark" series of podcasts).
Intrigued, I also watched the (excellent) video by Dreams of Autonomy:

https://www.youtube.com/watch?v=aghxkpyRVDY&pp=ygUGem94aWRl

OK... I was convinced.

I saw that it worked for `bash` and friends - not obviously for `tcsh` (I
tried and failed). `tcsh` was not worth giving up for `autojump` or `zoxide`.

So I decided to make a version of `autojump`/`zoxide` that works in `tcsh`.
I mean... how hard can it be? (Also, it's much more fun to write your own
code than debug someone elses).

I wrote it in pure C++-17 - it took a weekend. It worked and I was happy.

One of my colleagues is a fan of Rust - and I decided to write a program in
Rust to see what it was like - so this is that project - a Rust rewrite of
my version of `autojump`/`zoxide`.

`z-methoxy` uses the most recent access time (by z-methoxy) and the
number of times that directory has been used to weight its selection options -
paths that have been recently used have increased weight and
directories that are used many times have higher weights.

  - path name _fragments_ can be used (_e.g._ `cd p e git z` which will match
    "$HOME/Projects/external/git/zoxide")
  - The match for the path components is case insensitive
  - the path components fragments are matched in order
  - the last path component must match the last directory in the path
  - the columns in a `cdi` reponse are the time since last used (in days), the
    number of times the path has been used and then the actual path


TODO:
   - I will rewrite the data-store to use SQL/Limbo.
   - Aging old entries (deleting them from the store)

If you want to use it, make sure that `z-methoxy` is in your path

Say, for example:
```
$ cargo install --path .
$ set path = (~/.cargo/bin $path)
$ rehash
```

and add the following aliases (typically you would put them in your `~/.tcshrc` file):

```alias cd  'set zoch3_tmp="$cwd" ; chdir "`z-methoxy \!* | tail -1`" ; setenv OLD_DIR "$zoch3_tmp"'```

```alias cdi 'set zoch3_tmp="$cwd" ; chdir "`z-methoxy --show-matches \!* | grep -v ^# | fzf | z-methoxy --cut | tail -1`" ; setenv OLD_DIR "$zoch3_tmp"'```

As you can see, `z-methoxy` takes control of the `cd` command.
It also introduces an new alias, `cdi`, which allows `cd` to be used interactively (by
 using [`fzf`](https://github.com/junegunn/fzf) (so you will need to install that for `cdi` to work)).

