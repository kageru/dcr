# dcr – `dc` but in Rust
Sort of, at least. The goal here isn’t to replace `dc` at all. I just wanted my own RPN calculator.
I have some stupid ideas, but we’ll see about those.

Currently implemented (`xyz` are placeholders for popped stack values):
- `x y +`: pushes `x + y`. `-`, `*`, `/` work the same.
- `x y <`: pushes `1` if `x < y`, else pushes `0`. `>` and `=` work the same.
- `x y z ?`: if `z` is a nonzero value, push x, else push y
- `x p`: print `x`
- `f`: print the stack
- `q`: exit the program
- `c`: clear the stack
- `S`: push the current size of the stack
- `x y s`: store `x` in register `y`
- `x l`: load the value from register `x` and push it
- `x y r`: pop x, then push it `y` times. If `x` is a function, it is instead applied `y` times.
- `\x`: Put an `x` on the stack without executing it. `x` has to be a function that takes 1 or more arguments
- `x $`: pop `x` (a function) and apply it. It may pop any number of arguments it requires
- `x y $`: curry `x` (a function) with `y` (a value)

FAQ (answers to questions that I thought people might ask; the questions can be inferred by the reader):
- Whitespace is ignored except when it separates 2 numbers.
- All numbers are double precision floats.
- There are 256 registers, all pre-filled with zeros. They can only contain numbers.
- Reading from a register does not clear it.
- When used as register addresses, values are rounded if necessary.
- Anything after `#` is a comment and will be ignored.
