# kali

Kali is a simple functional programming language, developed during my time at the University of Cambridge.

## Features

Since I'm not particularly good at writing compilers, Kali is a very simple language. It has:

-   Common primitive types: `Int`, `Bool`, `Char`, `String`
-   Higher-order functions with currying
-   Algebraic data types
-   Pattern matching
-   Recursion
-   Type inference

## Execution Model

Kali is compiled to a simple stack machine, which is then interpreted. The stack machine has the following instructions:

-   `PushLiteral <value>`: Pushes a value onto the stack
-   `PushVariable <name>`: Pushes the value of a variable onto the stack
-   `Pop`: Pops a value from the stack
-   `BinaryOp <op>`: Pops two values from the stack, applies the binary operator `<op>`, and pushes the result
-   `UnaryOp <op>`: Pops a value from the stack, applies the unary operator `<op>`, and pushes the result
-   `ConditionalJump <addr>`: Pops a value from the stack, and jumps to `<addr>` if the value is false
-   `Jump <addr>`: Jumps to `<addr>`

Work is underway to compile Kali's stack machine IR using Cranelift, which will allow Kali to be compiled to native code.

## License

Kali is licensed under a dual license, either the Apache License, Version 2.0 or the MIT license, at your option. See [LICENSE-APACHE-2.0](LICENSE-APACHE-2.0) and [LICENSE-MIT](LICENSE-MIT) for details.
