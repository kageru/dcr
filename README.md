# dcr – `dc` but in Rust
Sort of, at least. The goal here isn’t to replace `dc` at all. I just wanted my own RPN calculator.
I have some lambda calculus ideas, but we’ll see about those.

Currently implemented:
- `+-/*`: basic math; pop 2 values, push 1
- `p`: pop and print 1 item
- `f`: print the stack without altering anything
- `c`: clear the stack
- `s`: pop 2 and store the first argument in a register specified by the second argument
- `l`: pop 1 argument and push the value of that register
- `q`: exit the program

FAQ (answers to questions that I thought people might ask; the questions can be inferred by the reader):
- If an operation takes multiple arguments, the “first” argument is the bottommost on the stack, e.g. `3 1 -` results in a stack containing `2` and `5 0 s` stores the number 5 in register 0, leaving an empty stack.
- Whitespace is ignored except when it separates 2 numbers.  
- All numbers are double precision floats.  
- There are 256 registers, all pre-filled with zeros.  
- Reading from a register does not clear it.  
- When used as register addresses, values are rounded if necessary.  
