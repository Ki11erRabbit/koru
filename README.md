# Koru Edit
A text editor written in my Rust and Guile Scheme
This editor is inspired by Emacs and Kakoune.

## Why Koru
Koru comes from Māori which is a loop or coil.
It refers to the silver fern frond.
The symbolism is that of new life, growth, yet familiarity.

## Features (WIP)
- [ ] Sane Syntax Highlighting
- [ ] Easy Theming
- [ ] Sane Keybinding Setups
- [ ] TUI and GUI Frontends
- [ ] Multiple Cursors
- [ ] Multiple Selections
- [ ] Multiple Modes (and doesn't need to be modal)
- [ ] Tabs
- [ ] Panes
- [ ] Completely Configurable in Koru Scheme
- [ ] Undo/Redo
- [ ] Color Schemes
- [ ] Client/Server Architecture
- [ ] Network Transparent Protocol



## Ideas on how to acomplish above goals
#### Sane Syntax Highlighting and Easy Theming
By default have a bunch of colors that must be implemented for a color scheme. 
This should then make it so that any plugin that requires colors can just pull it directly from the possible colors.
Syntax highlighting could be a minor mode that pulls from the required colors

#### Sane Keybinding Setups
There are different issues I have with Emacs and Vim with keybinding.
* Vim: Keybinding feels like an afterthought and is jank when using different keyboard layout
* Emacs: While better than Vim, you have to bind each key manually to a command. If using a non-qwerty layout, then you also have to unbind everything as well. This leads to a lot of work that I think is mostly unesessary
##### How to fix this problem
The idea is to have keybinding groups. 
Rather than binding key `h` to something like `CursorLeftChar` in every mode, you add a key to a group of keys that can be polled for. 
So instead of polling for a particular key, you can poll for a group.
For our example you would poll on `CursorLeftOne` to see if `h`, `Left`, or `C-P` were pressed.
The polling function could return a Union that is either a key or a key group if it falls into that group.
This should hopefully allow for new modes to add new keybindings without needing to poll for specific keys and make user configuration easier.

#### Completely Configurable in Koru Scheme
Emacs Lisp is jank and old. As such a more modern language like scheme could be more appropriate. 
Lisp in Emacs is also jank with there being multiple ways to run ELisp.

In Koru, there should only be one way to execute Koru Scheme. 
The one way should be to compile it at startup into a Rowan Class file and cache it.
Because we will be executing Rowan Bytecode, Koru Scheme should have roughly the same performance as Rowan. 
This also makes it possible to write configurations in Rowan if needed but favor Koru Scheme.

In fact, when Koru is distributed, it should come with only a few Koru Scheme files.
These files should provide a way for the user to choose their starting configuration.
The rest should just be the configuration files.
This way, the base editor it extremely barebones, but provide a framework for the user to create their config right away.
