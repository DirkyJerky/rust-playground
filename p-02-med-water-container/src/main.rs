use itertools::Itertools;

#[derive(Debug, PartialEq, Eq)]
struct Output {
    total: u64,
    left_idx: usize,
    right_idx: usize,
}

fn find_max(edges: Vec<u32>) -> Option<Output> {
    let mut lefts: Vec<usize> = Vec::with_capacity(edges.len() / 2);
    let mut rights: Vec<usize> = Vec::with_capacity(edges.len() / 2);

    let mut max_left: Option<u32> = None;
    for (idx, val) in edges.iter().enumerate() {
        if max_left.map_or(true, |max_left| max_left < *val) {
            max_left = Some(*val);
            lefts.push(idx);
        }
    }

    let mut max_right: Option<u32> = None;
    for (idx, val) in edges.iter().enumerate().rev() {
        if max_right.map_or(true, |max_right| max_right < *val) {
            max_right = Some(*val);
            rights.push(idx);
        }
    }

    dbg!(&lefts);
    dbg!(&rights);

    lefts
        .iter()
        .cartesian_product(rights.iter())
        .filter(|(left, right)| left < right)
        .map(|(left_idx, right_idx)| Output {
            left_idx: *left_idx,
            right_idx: *right_idx,
            total: edges[*left_idx].min(edges[*right_idx]) as u64 * (*right_idx - *left_idx) as u64,
        })
        .max_by_key(|output| output.total)
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_success() {
        let cases = [
            (
                vec![1, 1],
                Some(Output {
                    left_idx: 0,
                    right_idx: 1,
                    total: 1,
                }),
            ),
            (
                vec![1, 8, 6, 2, 5, 4, 8, 3, 7],
                Some(Output {
                    left_idx: 1,
                    right_idx: 8,
                    total: 49,
                }),
            ),
            (vec![1], None),
            (vec![], None),
            (
                vec![1, 2, 3, 4, 5, 4, 3, 2, 1],
                Some(Output {
                    left_idx: 2,
                    right_idx: 6,
                    total: 12,
                }),
            ),
            (
                vec![5, 4, 3, 2, 1, 2, 3, 4, 5],
                Some(Output {
                    left_idx: 0,
                    right_idx: 8,
                    total: 40,
                }),
            ),
        ];
        for (input, output) in cases {
            assert_eq!(find_max(input), output);
        }
    }
}
