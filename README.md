# set_builder

A procedural macro to create Iterators over a set defined by Haskell-inspired [set-builder notation](https://wikipedia.org/wiki/Set-builder_notation).

It should be noted that these "sets" are not true sets in the sense that there is no guarantee that the members (elements) of the set are unique.

# Syntax
## Complex Set
```rs
//  The name(s) of the binding(s)
//               │
//               │       ┌── Set(s) that evaluate into types implementing `IntoIterator`.
//               │       │
//       ┌───────┤       │        ┌─ The optional predicate, evaluates to `bool`
//       ▼       ▼       ▼        ▼
set! { ident : ident <- expr $(, expr?) }
set! { ($(ident),*) : $(ident <- expr),* $(, expr)? }
```

## Simple Enumeration Set
This is only provided for mathematical parity and returns arrays rather than Iterators,
array syntax `[...]` should always be preferred to this.

It is noteworthy that this will only work with literals.
If you wish to use identifiers, please use array syntax instead.

```rs
//         ┌─ Literal(s) to put in the set
//         ▼
set! { $(literal),* }
```

# Examples
```rs
use set_builder::set;

// Single-binding set with a predicate
let set = set! { x : x <- [1, 2, 3], *x > 1 };
assert_eq!(set.collect::<Vec<_>>(), [2, 3]);

// Cartesian product without a predicate
let set = set! { (x, y) : x <- [1, 2], y <- [3, 4] };
assert_eq!(set.collect::<Vec<_>>(), [(1, 3), (1, 4), (2, 3), (2, 4)]);

// Simple enumeration
let set = set! { 1, 2, 3, 4, 5 };
assert_eq!(set, [1, 2, 3, 4, 5]);
```
