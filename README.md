# dcr – `dc` but in Rust
Sort of, at least. The goal here isn’t to replace `dc` at all. I just wanted my own RPN calculator.
I have some stupid ideas, but we’ll see about those.

Currently implemented (`xyz` are placeholders for popped stack values):
- `x y +`: pushes `x + y`. `-`, `*`, `/`, `%` (modulo) work the same.
- `x y <`: pushes `1` if `x < y`, else pushes `0`. `>` and `=` work the same.
- `x y z ?`: if `x` is a nonzero value, push y, else push z
- `x p`: print `x`
- `f`: print the stack
- `q`: exit the program
- `c`: clear the stack
- `S`: push the current size of the stack
- `x y s`: store `x` in register `y`
- `x l`: load the value from register `x` and push it
- `(asdf)`: put the identifier `asdf` on the stack. It can be used to store functions/values with `s` or load/apply them.
- `x y r`: pop x, then push it `y` times. If `x` is a function, it is instead applied `y` times.
- `\x`: Put an `x` on the stack without executing it. `x` has to be a function that takes 1 or more arguments
- `x $`: pop `x` (a function) and apply it. It may pop any number of arguments it requires
- `x y @`: curry `x` (a function) with `y` (still looking for a better operator than `@`). Currying starts from the last argument, so the order is consistent with regular application. e.g. `\/ 2 @` creates a partial that will divide its argument by 2. Anything can be curried with anything, and a function can be curried any number of times. Before it is applied, all curried arguments are pushed on the stack in reverse order, i.e. `\+ 2@ 3@ 4@ 5@` will, if applied, push `5 4 3 2` before executing `+`, resulting in a stack of `5 4 5`.
- `x y |`: compose two functions, mainly useful when you want to store the result. When applying `a x y | $`, the result is identical to `y a x $ $`, i.e. `y(x(a))`, so functions are applied left to right.

### Function mode
Expressions within `{}` are in function mode. While in function mode, all operations except curry and compose are lazy, all values will be curried automatically, and all functions are composed, e.g. `f(x) = (x + 1) * 2` could be written as `\+1@\*2@|` normally or `{+1*2}` using function mode. Well-formedness of the braces is not enforced, and function mode is cleared at the end of each line.  
A more realistic and useful example is this implementation of a `min()` function, returning the smaller of 2 numbers:
```rs
# Imperative pseudocode for reference:
# fn min(a, b) {
#   registers[0] = a;
#   registers[1] = b;
#   return if registers[0] < registers[1] { registers[0] } else { registers[1] }
# }
# Regular mode
\s0@ \s1@ | \l1@ | \l0@ | \< | \l1@ | \l0@ | \? | (min)s
# Function mode
{ s0 s1 l1 l0 < l1 l0 ? }(min)s
```

On a technical level, all functions are replaced with their escaped (e.g. `\+`) counterparts, all numbers are implicitly followed by the curry operator `@`, and all functions after the first 2 are preceded by the compose operator `|`, also, a compose operator is added at the closing `}` it at least 2 functions were called in the block.  
Any identifier is loaded and composed implicitly, as can be seen in this implementation of `average` using an existing `sum` function (reminder: `S` pushes the current size of the stack):
```rs
{ S s0 (sum) l0 / }(average)s
```

### FAQ (answers to questions that I thought people might ask; the questions can be inferred by the reader):
- Whitespace is ignored except when it separates 2 numbers.
- All numbers are double precision floats.
- There are 256 registers (0-255), all pre-filled with zeros. They can only contain numbers.
- Reading from a register does not clear it.
- When used as register addresses, values are rounded if necessary.
- Anything after `#` is a comment and will be ignored.
