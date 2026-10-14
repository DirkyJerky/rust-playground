use itertools::Itertools;

#[derive(Debug)]
pub struct TrappedWaterProblem {
    topography: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl TrappedWaterProblem {
    pub fn new(topography: Vec<u32>) -> Self {
        Self { topography }
    }

    pub fn solve(&self) -> u32 {
        // 1. Setup left defining edge list.
        // Marks all edges with `Direction::Left`
        let mut found_max_left_size: u32 = 0;
        let left_edges_iter = self
            .topography
            .iter()
            .enumerate()
            .filter_map(|(idx, height)| {
                if *height > found_max_left_size {
                    found_max_left_size = *height;
                    Some((idx, height, Direction::Left))
                } else {
                    None
                }
            });

        // 2. Setup right defining edge list, which is naturally reversed.
        // Marks all edges with `Direction::Right`
        let mut found_max_right_size: u32 = 0;
        let right_edges_iter =
            self.topography
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(idx, height)| {
                    if *height > found_max_right_size {
                        found_max_right_size = *height;
                        Some((idx, height, Direction::Right))
                    } else {
                        None
                    }
                });

        // 3. Combine the iterators, reversing the right iterator so the final iterator is in index order
        let combined_iter =
            left_edges_iter.chain(right_edges_iter.collect::<Vec<_>>().into_iter().rev());

        let mut total: u32 = 0;

        // 4. Iterate over every possible interval of defining edges, which define the height of the contained pools
        for [(idx, i_height, i_direction), (jdx, j_height, j_direction)] in
            combined_iter.array_windows()
        {
            // 4a. Get the pool height, which depend on the source (left or right) the edge came from
            let pool_height = match (i_direction, j_direction) {
                (Direction::Left, Direction::Left) => i_height,
                (Direction::Left, Direction::Right) => i_height, // either works, for they will be the same value
                (Direction::Right, Direction::Right) => j_height,
                (Direction::Right, Direction::Left) => unreachable!(),
            };

            // 4b. Sum up the depth map over the defined interval.
            total += self.topography[idx..=jdx]
                .iter()
                .map(|height| (*pool_height).saturating_sub(*height))
                .sum::<u32>()
        }

        // 5. Return the total
        total
    }
}
