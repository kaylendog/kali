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
    #context align(right, counter(page).display())
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

==== Integers

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

Examples with non-Latin characters and emoji:

```kali
let greeting = "こんにちは世界"
let city = "Москва"
let mood = "Feeling great! 😊"
```

=== Composite Types

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

Fields on a record can be accessed using the `.` operator.

```kali
let point = { x: 10, y: 20 }
let x = point.x
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

The `Never` type is a special data type that has no values. It is used to represent a computation that never completes, or a value that can never be produced. It is often used to represent type errors, or to signal that type checking has failed.

> *Note:* The `Never` type is a theoretical type used internally by the compiler and type checker. It cannot be written or referenced directly in Kali source code. Programmers will never need to annotate a variable or function as `Never`, nor can a value of type `Never` be constructed or matched in user code.

== Control Flow

=== If Expressions

Kali uses `if` expressions for conditional logic. Since Kali is a functional language, every `if` must have an accompanying `else` branch to ensure that the expression always produces a value.

The basic syntax is:

```kali
let result = if condition {
  expr_if_true
} else {
  expr_if_false
}
```

Both the `if` and `else` branches must return a value of the same type.

You can chain multiple conditions using `else if`:

```kali
let sign = if x > 0 {
  "positive"
} else if x < 0 {
  "negative"
} else {
  "zero"
}
```

Because `if` is an expression, it can be used anywhere a value is expected, such as in variable bindings or function arguments.

=== Pattern Matching

Pattern matching in Kali is inspired by Rust's powerful and expressive match expressions. It allows you to match values against patterns and execute code based on which pattern matches.

The basic syntax is as follows:

```kali
match value {
  pattern1 -> expr1,
  pattern2 -> expr2,
  _ -> expr_default,
}
```

Each arm consists of a pattern, the `->` symbol, and an expression to evaluate if the pattern matches. The `_` pattern is a catch-all that matches any value not matched by previous patterns.

For example, matching on a sum type:

```kali
type Shape = Circle(Int) | Rectangle(Int, Int)

let area = match shape {
  Circle(r) => 3.14 * r * r,
  Rectangle(w, h) => w * h,
}
```

You can also use pattern matching with primitive types:

```kali
let result = match x {
  0 => "zero",
  1 => "one",
  _ => "many",
}
```

Patterns can destructure tuples, arrays, and records:

```kali
let point = (1, 2)
let description = match point {
  (0, 0) => "origin",
  (x, y) => "point",
}
```

Pattern matching in Kali is exhaustive: the compiler will warn if not all possible cases are covered, unless a catch-all `_` pattern is provided.

== Functions

=== Higher-order Functions

=== Currying

Currying is the process of transforming a function that takes multiple arguments into a series of functions that each take a single argument.

```kali
let add = x: Int, y: Int => x + y
let add1 = add 1
```

= Type System

Kali's type system is heavily inspired by that of TypeScript and OCaml, and is designed to be both expressive and flexible.

== Traits

== Type Inferrence

= Memory Management

= Modules

== Import and Export

Kali uses an import/export system that allows code to be organized into reusable modules.

=== Exporting

To make functions, types, or values available outside a module, use the `export` keyword:

```kali
export let PI = 3.1415;

export fn area_circle(r: Float): Float {
  PI * r * r
}

export type Point = { x: Int, y: Int };
```

You can export multiple items from a module. Only exported items are accessible from other modules; non-exported items are private to the module.

=== Importing

To use exported items from another module, use the `import` keyword with path-based syntax. You can import all exported symbols from a module into scope:

```kali
import example::*;
```

Or import a specific item:

```kali
import example::add_one;
```

You can also import from nested modules or the standard library:

```kali
import std::result::result;
import std::vec::vec;
```

This system allows Kali code to be modular, maintainable, and easy to share, using a concise and familiar path-based import syntax.

= Standard Library

Kali comes with a standard library that provides a wide range of functionality, including data structures, algorithms, and utilities.

= Kali Virtual Machine

Kali is designed to run on a virtual machine, namely the Kali Virtual Machine (KVM). This stack-based bytecode interpreter is capable of executing compiled Kali programs using a somewhat simple instruction set.

#table(
  columns: 3,
  "Name", "Opcode", "Description",
  "NOP", "0x00", "No operation"
)

= Interoperability

= Tooling

== Compiler

The compiler is a collection of tools used to compile Kali source code into its bytecode representation.

== Parser

The parser is a fully-fledged LALR(1) parser generated using the `lalrpop` parser generator. It constructs the AST from the source code directly with no intermediate representation.

== Passes

== Formatter

== Package Manager

= Appendix
