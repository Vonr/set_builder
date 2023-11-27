# set_builder

A procedural macro to create Iterators over a set defined by Haskell-inspired [set-builder notation](https://wikipedia.org/wiki/Set-builder_notation).

It should be noted that these "sets" are not true sets in the sense that there is no guarantee that the members (elements) of the set are unique.

# Syntax
## Complex Set
```rust,ignore
//   The pattern of the binding(s)
//               │
//   Mapping     │      ┌── Expression that evaluate into types implementing `IntoIterator`.
//  expression   │      │
//      │        │      │            ┌─ Predicate that evaluates to `bool`
//      ▼        ▼      ▼            ▼
set![ expr : $($(pat <- expr) | $(, expr)),* ]
```

## Simple Enumeration Set
This is only provided for mathematical parity and returns arrays rather than Iterators,
array syntax `[...]` should always be preferred to this.

```rust,ignore
//         ┌─ Literal(s) to put in the set
//         ▼
set![ $(expr),* ]
```

# Examples
```rust
use set_builder::set;

// Single-binding set with a predicate
let set = set![ x * 2 : x <- [1, 2, 3], *x > 1 ];
assert_eq!(set.collect::<Vec<_>>(), [4, 6]);

// Cartesian product without a predicate
let set = set![ (x, y) : x <- [1, 2], y <- [3, 4] ];
assert_eq!(set.collect::<Vec<_>>(), [(1, 3), (1, 4), (2, 3), (2, 4)]);

// Simple enumeration
let set = set![ 1, 2, 3, 4, 5 ];
assert_eq!(set, [1, 2, 3, 4, 5]);
```
