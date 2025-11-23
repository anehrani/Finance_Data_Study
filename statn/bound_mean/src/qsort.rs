// Rust has built-in sort methods which are efficient (Timsort-like or pdqsort).
// We don't need to implement QuickSort manually.
// This module is kept for structure compatibility if needed, or can be empty.

#[allow(dead_code)]
pub fn qsortd(data: &mut [f64]) {
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
}
