# Supported environments

Basic functionality tested on:
### OS:
- Arch linux

### Shells:
- zsh
- bash
- sh (when symlinked)

### Terminals:
- st
- urxvt

# Known failure modes
- sh (heirloom, archaic)
    - Doesn't work unless used as a login shell because .profile is not sourced
      which results in no prompt recognition.
- ssh
    - Prompt recognition not implemented

# Known bugs
- Segfault on resize when `new_cols > old_cols` and `new_rows < old_rows`
