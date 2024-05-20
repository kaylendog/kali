#set text(
  font: "Noto Sans",
  size: 11pt
)
#set page(
  paper: "a4",
  margin: (x: 2.5cm, y: 2.5cm),
)
#set par(
  justify: true,
  leading: 0.52em,
)

#set text(size: 32pt)
#heading(outlined: false, "Kali")

#set text(size: 16pt)
Specification - Version 1.0

#align(bottom + right, [#datetime.today().display("[day padding:none] [month repr:long] [year]")])

#pagebreak()

#set heading(numbering: "1.1.1")
#set text(size: 11pt)
#set page(
  footer: [
    #align(right, counter(page).display())
  ]
)

#show outline.entry.where(
  level: 1
): it => {
  v(11pt, weak: true)
  strong(it)
}

#outline(
  indent: 1cm,
)

#pagebreak()

= Introduction

Kali is a simple, elegant, and powerful programming language. It is designed to be easy to learn and use, while still being powerful enough to handle complex tasks.

== Motivations 

== Existing Languages

Kali takes inspiration from a variety of existing programming languages, including:
- Rust
- TypeScript
- OCaml

== Design Goals


= Syntax

== Data Types

=== Primitives

Like most programming languages, Kali has support for a variety of primitive data types:

- Integers
- Floats
- Booleans
- Strings

==== Integers <integers>

Integers are whole numbers, and can be positive or negative.

```kali
let x = 10
let y = -5
```

==== Floats

Floats are numbers with a decimal point, and can be positive or negative.

```kali
let x = 10.5
let y = -5.5
```

Kali uses the IEEE 754 standard for floating point arithmetic.

==== Booleans

Booleans are a data type that can have one of two values: `true` or `false`.

```kali
let x = true
let y = false
```

==== Strings

Strings are a sequence of characters, and are enclosed in double quotes.

```kali
let x = "Hello, world!"
```

All strings in Kali are UTF-8 encoded.

=== Composite

Kali has support for four composite data types:

- Product types
- Sum types
- Arrays
- Records

Composite data types are used to represent complex data structures, and are a fundamental part of the language.

Users can compose their own data types using these composite data types, and use them to build complex data structures.

```kali
type Person = (String, Int, Bool)
type Point = { x: Int, y: Int }
type Shape = Circle(Int) | Rectangle(Int, Int)
type List = Int[]
```

==== Product Types

Product types, also known as tuples, are a composite data type that can hold multiple values of different types. They are defined using the `()` syntax, with each value separated by a comma.

```kali
let person = ("John", 30, true)
```

The syntax for a tuple is as follows:

```ebnf
tuple = "(" type { "," type } ")"
```

The unit type is a special case of the tuple type, with no elements.

```kali
let unit = ()
```

==== Sum Types

Sum types are a composite data type that can hold one of several different values. They are defined using the `|` syntax, with each value separated by a pipe.

```kali
type Shape = Circle(Int) | Rectangle(Int, Int)
```

Sum types support three types of variant: simple, tuple, and record.

The syntax for a sum type is as follows:

```ebnf
sum_type =  sum_type_variant ("|" sum_type_variant)*
sum_type_variant = type ("(" type ("," type)* ")")
```

==== Arrays

Arrays are a composite data type that can hold multiple values of the same type. They are defined using the `[]` syntax, with each value separated by a comma.

```kali
let numbers = [1, 2, 3, 4, 5]
```

```ebnf
array_literal = "[" expr ("," expr)* "]"
array_access = expr "[" expr "]"
```

==== Records

Records are a convenient way to define named composite data types.

```kali
type Point = { x: Int, y: Int }
```

The syntax for a record is as follows:

```ebnf
record = "{" field { "," field } "}"
field = identifier ":" type
```

=== Special

There are a few special data types in Kali, which are used to represent special values.

- Unit
- Never

==== Unit

The unit type is a special data type that has only one value, also called `unit`. It is used to represent the absence of a value, or a value that is not interesting.

```kali
let unit = "()'
```

==== Never

The never type is a special data type that has no values. It is used to represent a computation that never completes, or a value that can never be produced.

It is often used to represent type errors, or to signal that type checking has failed.

```kali
type Point = { x: Int, y: Int }
```

Fields on a struct can be accessed using the `.` operator.

```kali
let point = { x: 10, y: 20 }
let x = point.x
```

== Variables

== Control Flow

== Pattern Matching

Pattern matching is a powerful feature that allows you to destructure complex data types and extract their values.

== Functions

=== Higher-order Functions

=== Currying

Currying is the process of transforming a function that takes multiple arguments into a series of functions that each take a single argument.

```kali
let add = x: Int, y: Int => x + y
let add1 = add 1
```

=== Closures

Closures are a special kind of function that can capture variables from their surrounding environment.

= Type System

Kali's type system is heavily inspired by that of TypeScript and OCaml, and is designed to be both expressive and flexible.

== Traits

== Type Inferrence

= Execution Model

Kali is a byte-code compiled language, which means that it is compiled to an intermediate representation that is then executed by a virtual machine.

This section describes an abstract execution model for Kali programs, and how they should be executed by a Kali virtual machine.

== Kali Virtual Machine

The Kali virtual machine is a stack-based virtual machine that executes Kali byte-code. It is designed to be simple, efficient, and easy to implement.

=== Stack Layout

Kali's stack is laid out similar to that of C. The stack grows downwards, with the top of the stack being at a lower memory address than the bottom of the stack.

=== Instructions

The stack machine supports a wide range of instructions, including arithmetic, logical, and control flow instructions.

Instructions may have at most one immediate operand, which can vary in size depending on the instruction.

==== Arithmetic

Kali has full support for arithemtic addition, subtraction, multiplication, and division with both integers and floats. As described in @integers, all integers are signed.

The following stack machine instructions are supported:

- `iadd`: Add two integers
- `isub`: Subtract two integers
- `imul`: Multiply two integers
- `idiv`: Divide two integers
- `imod`: Modulus of two integers
- `ineg`: Negate an integer
- `ipow`: Power of two integers
- `fadd`: Add two floats
- `fsub`: Subtract two floats
- `fmul`: Multiply two floats
- `fdiv`: Divide two floats
- `fmod`: Modulus of two floats
- `fneg`: Negate a float
- `fpow`: Power of two floats
- `fsqrt`: Square root of a float

==== Logical

The following stack machine instructions are supported on booleans:

- `land`: Logical AND
- `lor`: Logical OR
- `lnot`: Logical NOT
- `lxor`: Logical XOR

The following stack machine instructions are supported on integers and floats:

- `ieq`: Integer equality
- `ineq`: Integer inequality
- `ilt`: Integer less than
- `ile`: Integer less than or equal
- `igt`: Integer greater than
- `ige`: Integer greater than or equal
- `feq`: Float equality
- `fneq`: Float inequality
- `flt`: Float less than
- `fle`: Float less than or equal
- `fgt`: Float greater than
- `fge`: Float greater than or equal

==== Control Flow

- `jmp`: Jump to an address in the current block
- `jz`: Jump if zero
- `jnz`: Jump if not zero
- `call`: Call a function
- `ret`: Return from a function
- `halt`: Halt the virtual machine
- `nop`: No operation

==== Stack Manipulation

- `ipush <int>`: Push an integer onto the stack
- `fpush <float>`: Push a float onto the stack
- `bpush <bool>`: Push a boolean onto the stack
- `ld <addr>`: Load a value from memory and push it onto the stack
- `ipop`: Pop a value from the stack

== Bytecode

Each instruction in the Kali byte-code is represented by a single byte, with an optional immediate operand, varying in size depending on the instruction.

== Memory Management

== Native Targets

Kali's byte-code can be compiled to a variety of native targets. Since the code generator uses Cranelift, it can target a wide range of platforms, including x86, ARM, and WebAssembly.

= Modules

= Standard Library

Kali comes with a standard library that provides a wide range of functionality, including data structures, algorithms, and utilities.

= Interoperability

= Tooling

== Compiler

== Formatter

== Package Manager

= Appendix
