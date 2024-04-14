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

== Design Goals


= Syntax

== Data Types

=== Primitives

Kali has support for six primitive data types:

- Integers
- Floating-point numbers
- Booleans
- Strings
- Characters

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

== Variables

== Control Flow

== Functions

= Type System

== Traits

== Type Inferrence

= Memory Management

= Modules

= Standard Library

Kali comes with a standard library that provides a wide range of functionality, including data structures, algorithms, and utilities.

= Interoperability

= Tooling

== Compiler

== Formatter

== Package Manager

= Appendix
