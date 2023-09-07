use std::collections::HashSet;

pub fn deduplicate<T>(input: &[T]) -> Vec<T> where T: Clone + Eq + std::hash::Hash {
    let mut seen: HashSet<&T> = HashSet::with_capacity(input.len());
    let mut output: Vec<T> = Vec::with_capacity(input.len());
    for item in input {
        if !seen.contains(item) {
            seen.insert(item);
            output.push(item.clone());
        }
    }
    output
}

#[test]
fn test_deduplicate() {
    assert_eq!(deduplicate(&[1, 2, 3]), [1, 2, 3]);
    assert_eq!(deduplicate(&[1, 2, 2, 3, 3]), [1, 2, 3]);
    assert_eq!(deduplicate(&[1, 1, 1, 1, 1]), [1]);

    let input: Vec<i32> = (1..=5000).chain(1..=5000).collect();
    let expected: Vec<i32> = (1..=5000).collect();
    assert_eq!(deduplicate(&input), expected);

    let empty: Vec<i32> = Vec::new();
    assert_eq!(deduplicate(&empty), empty);

    assert_eq!(deduplicate(&["a", "b", "c", "a", "b"]), ["a", "b", "c"]);
    assert_eq!(deduplicate(&["hello", "world", "hello"]), ["hello", "world"]);

    assert_eq!(deduplicate(&[1, 3, 2, 3, 4, 1]), [1, 3, 2, 4]);

    assert_eq!(deduplicate(&[3, 1, 2]), [3, 1, 2]);
}

