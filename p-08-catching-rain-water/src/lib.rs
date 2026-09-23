mod trapped_water_problem;

use trapped_water_problem::TrappedWaterProblem;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cases() {
        let cases: [(Vec<u32>, u32); _] = [
            (vec![], 0),
            (vec![1], 0),
            (vec![1, 1], 0),
            (vec![1, 2], 0),
            (vec![1, 1, 1], 0),
            (vec![1, 0, 1], 1),
            (vec![2, 0, 1], 1),
            (vec![2, 0, 2], 2),
            (vec![0, 2, 0, 2, 0], 2),
            (vec![0, 2, 1, 2, 0], 1),
            (vec![0, 1, 2, 4, 1, 8, 1, 4, 0], 6),
            (vec![0, 1, 0, 2, 1, 0, 1, 3, 2, 1, 2, 1], 6),
            (vec![100, 0, 50, 25, 25, 100], 300),
        ];

        for (input, output) in cases {
            assert_eq!(
                TrappedWaterProblem::new(input.clone()).solve(),
                output,
                "{input:?} -> {output:?}"
            )
        }
    }
}
