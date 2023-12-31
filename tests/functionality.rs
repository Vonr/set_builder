#![recursion_limit = "1024"]

use set_builder::set;

#[test]
fn full_readme() {
    let set = set![ x * 2 : x <- [1, 2, 3], *x > 1 ];
    assert_eq!(set.collect::<Vec<_>>(), [4, 6]);
}

#[test]
fn full_unpredicated() {
    let set = set![ (x, y, z) : x <- [1, 2, 3], y <- [4, 5, 6], z <- [7, 8, 9] ];

    assert_eq!(
        set.collect::<Vec<_>>(),
        [
            (1, 4, 7),
            (1, 4, 8),
            (1, 4, 9),
            (1, 5, 7),
            (1, 5, 8),
            (1, 5, 9),
            (1, 6, 7),
            (1, 6, 8),
            (1, 6, 9),
            (2, 4, 7),
            (2, 4, 8),
            (2, 4, 9),
            (2, 5, 7),
            (2, 5, 8),
            (2, 5, 9),
            (2, 6, 7),
            (2, 6, 8),
            (2, 6, 9),
            (3, 4, 7),
            (3, 4, 8),
            (3, 4, 9),
            (3, 5, 7),
            (3, 5, 8),
            (3, 5, 9),
            (3, 6, 7),
            (3, 6, 8),
            (3, 6, 9)
        ]
    );
}

#[test]
fn full_predicated() {
    let set = set![ (x, y, z) : x <- [1, 2, 3], *x > 1, y <- [4, 5, 6], z <- [7, 8, 9], *z != 8 ];

    assert_eq!(
        set.collect::<Vec<_>>(),
        [
            (2, 4, 7),
            (2, 4, 9),
            (2, 5, 7),
            (2, 5, 9),
            (2, 6, 7),
            (2, 6, 9),
            (3, 4, 7),
            (3, 4, 9),
            (3, 5, 7),
            (3, 5, 9),
            (3, 6, 7),
            (3, 6, 9)
        ]
    );
}

#[test]
fn full_pattern() {
    let set = set![ (*a, *b) : (i, a) <- [1, 2, 3].iter().enumerate(), (j, b) <- [10, 20, 30].iter().enumerate(), i != j ];
    assert_eq!(
        set.collect::<Vec<_>>(),
        [(1, 20), (1, 30), (2, 10), (2, 30), (3, 10), (3, 20)]
    );
}

#[test]
fn many() {
    let set = set![ (a, b, c, d, e, f, g, h, i, j, k)
        : a <- [1, 2, 3],
          b <- [1, 2, 3],
          c <- [1, 2, 3],
          d <- [1, 2, 3],
          e <- [1, 2, 3],
          f <- [1, 2, 3],
          g <- [1, 2, 3],
          h <- [1, 2, 3],
          i <- [1, 2, 3],
          j <- [1, 2, 3],
          k <- [1, 2, 3],
    ];
    assert_eq!(set.count(), 3_usize.pow(11));
}

#[test]
fn enumeration() {
    let set = set![1, 2, 3, 4];
    assert_eq!(set, [1, 2, 3, 4]);
}
