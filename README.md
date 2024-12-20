
For many years, off and on, I'd been trying to get `autojump` to work
with `tcsh`.  I never managed it.  `autojump` was useful (for others)
because it remembered the directories that you had visited, so that,
at a later date you could use that information to visit the directory
again using less memory and key strokes.

Recently I heard a review of [zoxide](https://github.com/ajeetdsouza/zoxide) on "Linux Dev Time" (or one of the
"Linux After Dark" series of podcasts). Intrigued, I also watched
the (excellent) video by Dreams of Autonomy:

https://www.youtube.com/watch?v=aghxkpyRVDY&pp=ygUGem94aWRl

I saw that it worked for `bash` and friends - not really for `tcsh`.
`tcsh` was not worth giving up for `autojump` or `zoxide`.

So I decided to make a version of autojump/zoxide that works in tcsh.
I mean... how hard can it be?

I wrote it in pure C++-17 - it took a weekend. It worked and I was happy.

One of my colleagues is a fan of Rust - and I decided to write a
program in Rust to see what it was like - so this is that project - a
Rust rewrite of my version of autojump/zoxide.

If you want to use it, make sure that `z-methoxy` is in your path
and add the following aliases (typically one would put them in one's `~/.tcshrc` file):

```alias cd  'set zoch3_tmp="$cwd" ; chdir "`z-methoxy \!* | tail -1`" ; setenv OLD_DIR "$zoch3_tmp"'```

```alias cdi 'set zoch3_tmp="$cwd" ; chdir "`z-methoxy --show-matches \!* | grep -v ^# | fzf | z-methoxy --cut | tail -1`" ; setenv OLD_DIR "$zoch3_tmp"'```

As you can see, `z-methoxy` takes control of the `cd` command.
It also introduces an new alias, `cdi`, which allow `cd` to be used interactively (by using `fzf` (so you will need to install that for `cdi` to work)).

