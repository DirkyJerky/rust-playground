use itertools::merge;

fn median_of_two_sorted_lists(list_1: Vec<u32>, list_2: Vec<u32>) -> Option<f32> {
    let size = list_1.len() + list_2.len();
    if size == 0 {
        return None;
    }

    let mut it = merge(list_1, list_2);

    let is_even_size = size % 2 == 0;

    let mid = (size - 1) / 2; // truncates down if `size` is even

    let nth = it.nth(mid).expect("should have value");
    if is_even_size {
        Some((it.next().expect("should have value") + nth) as f32 / 2.)
    } else {
        Some(nth as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cases() {
        let cases: [(Vec<u32>, Vec<u32>, Option<f32>); _] = [
            (vec![], vec![], None),
            (vec![], vec![1], Some(1.)),
            (vec![1], vec![], Some(1.)),
            (vec![1], vec![2], Some(1.5)),
            (vec![1], vec![2, 3], Some(2.)),
            (vec![1, 3, 5, 7, 9], vec![2, 4, 6, 8], Some(5.)),
            (vec![1, 1, 1, 1, 5, 5, 5, 5, 5], vec![3], Some(4.)),
        ];

        for (input1, input2, output) in cases {
            assert_eq!(median_of_two_sorted_lists(input1, input2), output);
        }
    }
}
