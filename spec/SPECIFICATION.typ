// Noto Sans is vendored in spec/fonts (SIL OFL 1.1). Compile with:
//   typst compile --font-path spec/fonts spec/SPECIFICATION.typ
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

// Marks a feature the parser or compiler does not implement yet.
#let status(body) = block(
  inset: 6pt,
  radius: 3pt,
  fill: luma(240),
  width: 100%,
  text(size: 9pt, [*Status:* #body]),
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
- Go, for how line ends terminate items

== Design Goals

= Syntax

== Lexical Structure

=== Identifiers

Identifiers are any run of word characters, including Unicode letters, and underscores. Keywords and primitive type names are reserved.

```kali
let add = a b -> a + b
let μέσος = a b -> (a + b) / 2
```

=== Comments

Line comments start with `//`. Doc comments start with `///` and attach to the item that follows them.

```kali
// Adds two numbers.
let add = a b -> a + b

/// The ratio of a circle's circumference to its diameter.
let pi = 3
```

=== Items and Line Ends <items-and-line-ends>

A module is a sequence of items: `let` definitions, `type` aliases and `import`s. Items are terminated by the end of a line, in the same way as Go: a line end acts as a terminator when the last token on the line could end an item, such as an identifier, a literal or a closing bracket. A line ending in an operator, a comma or an opening bracket continues onto the next line. An explicit `;` is also accepted.

```kali
let x = fib (n - 1) +
  fib (n - 2)
let y = [1,
  2]
```

A consequence is that an operator must stay at the end of a line to continue it. Writing `+ fib (n - 2)` on its own line is an error.

== Data Types

=== Primitives

Kali has the following primitive data types. Type names are lowercase.

#table(
  columns: 3,
  [*Type*], [*Name*], [*Literals*],
  [Integer], `int`, `5`,
  [Float], `float`, `10.5`,
  [Boolean], `bool`, `true`,
  [String], `string`, `"Hello"`,
  [Unit], `()`, `()`,
)

==== Integers

Integer literals are signed 64-bit. They may use `_` as a separator and the prefixes `0x`, `0o`, `0b` and `0d`. A negative number is the negation operator applied to a literal, so `-5` and `n-1` both mean what they look like.

```kali
let x = 10
let y = -5
let big = 1_000_000
let mask = 0xff
```

==== Floats

Floats are numbers with a decimal point, and can be positive or negative.

```kali-planned
let x = 10.5
let y = -5.5
```

Kali uses the IEEE 754 standard for floating point arithmetic.

#status[The lexer has no float literal rule yet, so float literals are a parse error.]

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
type Person = (string, int, bool)
type Point = { x: int, y: int }
type Shape = Circle(int) | Rectangle(int, int)
type List = [int]
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
type Shape = Circle(int) | Rectangle(int, int)
```

Sum types support three types of variant: simple, tuple, and record.

The syntax for a sum type is as follows:

```ebnf
sum_type =  sum_type_variant ("|" sum_type_variant)*
sum_type_variant = type ("(" type ("," type)* ")")
```

#status[Named constructors are not implemented. The parser currently reads `a | b` as a structural union of two types and `a & b` as an intersection, TypeScript style, with `|` binding tighter than `&`. Whether Kali keeps structural unions, adds constructors, or both, is an open decision.]

==== Arrays

Arrays are a composite data type that can hold multiple values of the same type. They are defined using the `[]` syntax, with each value separated by a comma. The type of an array of `int` is written `[int]`.

```kali
let numbers = [1, 2, 3, 4, 5]
let empty = []
```

```ebnf
array_literal = "[" expr ("," expr)* "]" | "[]"
array_type = "[" type "]"
```

Arrays are taken apart with the `::` cons pattern, see @pattern-matching.

#status[Indexing with `expr[i]` is not implemented.]

==== Records

Records are a convenient way to define named composite data types.

```kali
type Point = { x: int, y: int }
```

The syntax for a record is as follows:

```ebnf
record = "{" field { "," field } "}"
field = identifier ":" type
```

Fields on a record can be accessed using the `.` operator.

```kali-planned
let point = { x: 10, y: 20 }
let x = point.x
```

#status[Record types and record patterns parse. Record literals and `.` field access do not.]

=== Special

There are a few special data types in Kali, which are used to represent special values.

- Unit
- Never

==== Unit

The unit type is a special data type that has only one value, also called `unit`. It is used to represent the absence of a value, or a value that is not interesting.

```kali
let unit = ()
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

```kali-planned
let sign = if x > 0 {
  "positive"
} else if x < 0 {
  "negative"
} else {
  "zero"
}
```

Because `if` is an expression, it can be used anywhere a value is expected, such as in variable bindings or function arguments.

#status[The parser accepts `if` without `else`; rejecting it is the type checker's job. `else if` is not parsed yet and must be written `else { if .. }`.]

=== Pattern Matching <pattern-matching>

Pattern matching in Kali is inspired by OCaml and Rust. It allows you to match values against patterns and execute code based on which pattern matches.

The basic syntax is as follows:

```kali
let result = match value {
  | pattern1 -> expr1
  | pattern2 -> expr2
  | _ -> expr_default
}
```

Each arm starts with `|`, followed by a pattern, the `->` symbol, and an expression to evaluate if the pattern matches. The braces close the match, so nested matches need no extra parentheses. The `_` pattern is a catch-all that matches any value not matched by previous patterns.

Patterns can destructure tuples, arrays, and records, and match on literals:

```kali
let describe = point -> match point {
  | (0, 0) -> "origin"
  | (x, 0) | (0, x) -> "on an axis"
  | (x, y) -> "point"
}

let sum = list -> match list {
  | [] -> 0
  | x :: rest -> x + sum rest
}

let origin = match point {
  | { x: 0, y: 0 } -> true
  | _ -> false
}
```

The available patterns are:

#table(
  columns: 2,
  [*Pattern*], [*Matches*],
  `_`, [anything, binding nothing],
  `x`, [anything, binding it to `x`],
  `0`, `true`, `"s"`, [a literal],
  `(p, q)`, [a tuple, element by element],
  `{ x: p }`, [a record, field by field],
  `[]`, [the empty array],
  `p :: q`, [a non-empty array, as its head and tail],
  `p | q`, [either pattern, an or-pattern],
)

Matching on a sum type:

```kali
type Shape = Circle(int) | Rectangle(int, int)

let area = shape -> match shape {
  | Circle(r) -> 3 * r * r
  | Rectangle(w, h) -> w * h
}
```

Pattern matching in Kali is exhaustive: the compiler will warn if not all possible cases are covered, unless a catch-all `_` pattern is provided.

#status[Constructor patterns such as `Circle(r)` depend on sum types, see above. Exhaustiveness checking is not implemented.]

== Functions

Functions are lambda expressions bound with `let`. Parameters are written before `->` separated by spaces, OCaml style, and a parameter with a type annotation is parenthesised.

```kali
let add = a b -> a + b
let scale = (factor: int) value -> factor * value
let fib = n -> match n {
  | 0 -> 0
  | 1 -> 1
  | n -> fib (n - 1) + fib (n - 2)
}
```

Application is by juxtaposition. An argument that is not a single token, such as `n - 1`, is parenthesised. Application binds tighter than every operator, see @operators, so `fib (n - 1) + fib (n - 2)` is two calls and an addition.

```kali
let three = add 1 2
```

#status[`f x, y` with commas is also accepted as an uncurried call. Whether to keep it is an open decision.]

=== Higher-order Functions

Functions are values and can be passed to and returned from other functions.

```kali
let map = f list -> match list {
  | [] -> []
  | x :: rest -> f x :: map f rest
}
```

=== Currying

Currying is the process of transforming a function that takes multiple arguments into a series of functions that each take a single argument. `add 1 2` is `(add 1) 2`, so a function may be applied to fewer arguments than it has parameters.

```kali
let add = a b -> a + b
let add1 = add 1
```

#status[Lambdas parse this way. Partial application is a type checker and core IR concern and is not implemented.]

== Operators <operators>

Operators group as in OCaml and Haskell: application binds tightest, prefix operators apply to a whole application, and `::` sits between arithmetic and comparison. From tightest to loosest:

#table(
  columns: 3,
  [*Operators*], [*Associativity*], [*Example*],
  [application `f x`], [left], [`f x y` is `(f x) y`],
  [prefix `-`, `+`, `!`, `~`], [], [`-f x` is `-(f x)`],
  [`**`], [right], [`a ** b ** c` is `a ** (b ** c)`],
  [`*`, `/`, `%`], [left], [],
  [`+`, `-`], [left], [],
  [`::`], [right], [`a + b :: c` is `(a + b) :: c`],
  [`<`, `<=`, `>`, `>=`], [left], [`x :: xs == l` is `(x :: xs) == l`],
  [`==`, `!=`], [left], [],
  [`&&`], [left], [],
  [`||`], [left], [],
  [`a b -> e`], [], [the body extends as far right as possible],
)

A prefix operator has no other meaning: `!` is logical not, never dereference, so `!f x` can only sensibly mean `!(f x)`. The formatter still parenthesises a compound operand, printing `-(f x)` and `!(f x)`, so the grouping is never misread.

```kali
let a = -f x + g y * 2 ** 3
let b = x :: rest == l && !done || p < q
```

In patterns, `::` binds tighter than `|`, as in OCaml and Rust: `a :: b | c` is `(a :: b) | c`.

#status[`&`, `|`, `^`, `<<`, `>>` and `@` are lexed but not parsed as expression operators.]

= Type System

Kali's type system is heavily inspired by that of TypeScript and OCaml, and is designed to be both expressive and flexible.

== Traits

== Type Inference

Types are inferred in the Hindley-Milner style. Annotations are optional and are used to constrain inference or document intent.

#status[The type checker predates the current AST and does not compile. It is being rewritten.]

= Memory Management

= Modules

== Import and Export

Kali uses an import/export system that allows code to be organized into reusable modules.

=== Exporting

To make functions, types, or values available outside a module, use the `export` keyword:

```kali-planned
export let pi = 3

export let area_circle = (r: float) -> pi * r * r

export type Point = { x: int, y: int }
```

You can export multiple items from a module. Only exported items are accessible from other modules; non-exported items are private to the module.

#status[`export` is lexed but not parsed.]

=== Importing

To use exported items from another module, use the `import` keyword with path-based syntax. Path segments are separated by `::`. You can import all exported symbols from a module into scope:

```kali
import example :: *
```

Or import a specific item, optionally under another name:

```kali
import example :: add_one
import example :: add_one as inc
```

Several items from one module can be imported in a list, and paths can be nested:

```kali
import std :: { result, vec as v }
import std :: result :: result
```

```ebnf
import = "import" tree
tree = identifier "::" tree
     | identifier [ "as" identifier ]
     | "*"
     | "{" tree { "," tree } "}"
```

#status[Imports parse. Resolving them to other files is not implemented.]

= Standard Library

Kali comes with a standard library that provides a wide range of functionality, including data structures, algorithms, and utilities.

= Compiler Pipeline

Kali compiles in four stages.

1. The parser produces an AST. Every node carries a span and a node id, and comments live in a side table rather than in the tree.
2. Name resolution and type inference run over the AST and record their results in tables keyed by node id. The tree itself is never rewritten.
3. The typed AST is elaborated into a small, explicitly typed core IR with no sugar. Pattern matching is compiled to decision trees and closures are converted here.
4. Core is lowered to `pliron`, an MLIR-style SSA IR, from which native code is generated.

#status[Only the parser exists. The earlier design, a stack-based bytecode interpreter, has been dropped.]

= Interoperability

= Tooling

== Compiler

The `kali` command line tool drives the pipeline. Its `debug` subcommands dump the output of each stage: `debug lex`, `debug parse` and `debug print`.

== Parser

The lexer is generated with `logos` and the parser is written by hand with the `chumsky` combinator library, using Pratt parsing for operators. Between the two, line ends are turned into item terminators as described in @items-and-line-ends.

== Passes

== Formatter

The formatter prints the AST back to source. It preserves comments and collapses runs of blank lines to one. Its output is stable: formatting formatted code changes nothing.

== Package Manager

= Appendix
