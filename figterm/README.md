# Dev Setup

After cloning...
1. Pull submodules `git submodule update --init --recursive`
2. Install `glibtool` using `brew install libtool`

Change files, run `make install` (with `make clean` first if you change
anything in `lib/`) and this will copy output binaries to your
`~/.fig/bin` directory.

# Known failure modes
- Doesn't support shells besides zsh, bash, fish. Should be relatively
    straightforward to support provided we can modify the prompt variables
    appropriately.
- SSH
    - Prompt recognition not fully implemented in config repo.

# How it works

Figterm is motivated by the question of how to extract the "edit buffer",
i.e. the command the user is currently typing, from a shell prompt. Some
shells like zsh provide hooks to get this directly on each user keypress.
For shells like bash, however, this is difficult.

Figterm solves this problem by injecting itself between the shell and
terminal emulator to construct a representation of what is displayed in
the user's terminal with contextual understanding of where the prompts,
etc. are to extract the edit buffer.

This depends on three primary components

1. _Pseudoterminals_ allow us to intercept the communication between a user's
    terminal emulator and shell.
2. _Libvterm_ allows us to reconstruct these input/output streams into
    a representation of a user's terminal screen.
3. _Shell integration_ allows us to augment this representation with
    information about the prompt location.

## Pseudoterminals

In the world of terminals there are several terms that are used in very
similar, and sometimes identical, ways. To clarify differences we define
a few below

- A **Terminal** = **TTY** is a text input/output environment, implemented
    as a file on disk that provides additional functionality beyond
    reading/writing that you would expect from a hardware terminal, e.g.
    it has a window size that can be set by the ioctl command `TIOCSWINSZ`
- A **Terminal Emulator** is an application that exposes a terminal to the
    user through a GUI. It translates escape codes sent from a shell.
    Terminal emulators support a set of escape codes to control state like
    cursor position, cell color, etc. This is often used synonymously with
    _terminal_.
- A **Pseudoterminal** = **PTY**, is a type of _TTY_, represented by a parent/child
    pair of files. The child provides a terminal like interface like a TTY
    that a controlled process can read from/write to (often a shell). The
    parent acts as a normal file that can be read from and written to to
    provide input to/ and access output from the controlled process.
- A **Shell** is a command line interpreter that provides a prompt for the
    user to input a command and then executes other subprocesses based on
    entered commands.

There are typically several built in hardware TTYs that have their
"parent" end connected to hardware and the "child" end connected to
software. On many linux distributions you can press `Ctrl+Alt+F1` to go to
hardware tty1.

PTYs, on the other hand, have their parent end connected to software as
well. Windowed programs like iTerm2, Terminal.app, Kitty, Alacritty, etc.
allocate their own pseudoterminals to emulate the behavior of a hardware
TTY.

Figterm, like many terminal emulators, allocates a PTY. The parent end is
connected to figterm itself and the child to a shell launched by figterm.
The shell launched by the user's terminal emulator (e.g. iTerm2) is
replaced by figterm, so that figterm is now the child end of iTerm2's PTY
and behaves like a shell to iTerm2 but a terminal emulator to bash.

## Libvterm

Libvterm is an open source terminal emulation library used by projects
like neovim. It exposes 3 layers of API:

1. A **parser** layer that interprets [escape codes](http://rtfm.etla.org/xterm/ctlseq.html).
2. A **state** layer that maintains a state of the terminal, including
    things like a cursor position.
3. A **screen** layer that maintains an array of characters, their color,
    boldness, etc., acting as a full terminal emulator

Figterm forwards input to iTerm2 to the shell's input, leaving an up
arrow press as an up arrow press. The output from the shell is forwarded
to iTerm2 to be rendered to the screen, but is also diverted to libvterm,
where we hook into the state layer of the API.

We provide our own screen API in `screen.c` that maintains information
about an array of characters on the screen, but instead of storing
information about cell color, we maintain information about whether a cell
is part of the prompt or not.

## Shell Integration

Libvterm exposes hooks from the parser in its state layer. If the built in
libvterm parser sequence isn't handled by the state layer already (e.g.
[CSI sequences](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences)
that move the cursor are handled by the state layer), we can hook into
this event and update our own state accordingly.

Figterm utilizes Operating System Commands (OSCs) similar to [those
implemented by iTerm2](https://iterm2.com/documentation-escape-codes.html).

In particular, Fig's config repo wraps all of the user's prompt's (`$PS1,
$RPS1, $PS2, $PS3`) with OSCs `OSC 697; StartPrompt ST` and `OSC 697;
EndPrompt ST `. This marks cells in figterm's representation of the screen
as part of the prompt, so we can exclude them from the edit buffer.

We also use the OSC `OSC 697; NewCmd ST` to denote the cursor position at
the most recent prompt (only `$PS1,$PS3`).

Other OSC's are used to inform the `figterm` process of the working
directory, PID, TTY, and SSH session of the shell it is controlling.
