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

=== Composite

Kali has support for four composite data types:

- Product types
- Sum types
- Arrays
- Structs

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
tuple = "(" type ("," type)* ")"
```

==== Sum Types

Sum types are a composite data type that can hold one of several different values. They are defined using the `|` syntax, with each value separated by a pipe.

```kali
type Shape = Circle(Int) | Rectangle(Int, Int)
```

The syntax for a sum type is as follows:

```ebnf
sum_type =  sum_type_variant ("|" sum_type_variant)*
sum_type_variant = type ("(" type ("," type)* ")"
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

==== Structs

Structs are a convenient way to define named composite data types.

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

= Memory Management

= Modules

= Standard Library

= Interoperability

= Tooling

== Compiler

== Formatter

== Package Manager

= Appendix
