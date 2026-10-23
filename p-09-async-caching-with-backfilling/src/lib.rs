mod external {
    use rand::SeedableRng;
    use rand::{RngExt, random_range};
    use rand_chacha::ChaChaRng;
    use tokio::time::{Duration, sleep};

    /// performs a call to a third party API
    pub async fn api_call(query_param: String) -> Result<String, Infallible> {
        // For sake of testing, return the query param itself as a response, after a delay

        println!("api query in: {query_param}");
        // let sleep_seconds: u64 = random_range(1..=5);
        sleep(Duration::from_secs(5)).await;

        println!("api result out: {query_param}");
        Ok(String::from(query_param))
    }

    use core::fmt::Debug;
    use core::marker::PhantomData;
    use std::convert::Infallible;
    use std::ops::RangeInclusive;

    struct TestSource {
        generations_left: usize,
        rng: ChaChaRng,
    }

    impl TestSource {
        const SEED: u64 = 123;
        const CHAR_RANGE: RangeInclusive<char> = 'a'..='z';
        const DELAY_RANGE: RangeInclusive<f32> = 0.1..=0.25;
        const NUM_GENERATIONS: usize = 100;

        fn new() -> Self {
            TestSource {
                generations_left: Self::NUM_GENERATIONS,
                rng: ChaChaRng::seed_from_u64(Self::SEED),
            }
        }

        async fn next_generation<T: From<char>>(&mut self) -> Option<T> {
            if let Some(new_generations_left) = self.generations_left.checked_sub(1) {
                self.generations_left = new_generations_left
            } else {
                return None;
            }

            let chr = self.rng.random_range(Self::CHAR_RANGE);
            let delay = self.rng.random_range(Self::DELAY_RANGE);

            sleep(Duration::from_secs_f32(delay)).await;

            println!("stream queries for: {chr}");
            Some(chr.into())
        }
    }

    pub struct Stream<T: Send + 'static + Debug> {
        _pd: PhantomData<T>,

        test_source: TestSource,
    }

    impl<T: Send + 'static + Debug + From<char>> Stream<T> {
        /// returns the next element of the stream
        pub async fn next(&mut self) -> Option<T> {
            // For sake of testing, return a deterministically random single character 'a-z' after a random delay
            self.test_source.next_generation().await
        }
    }

    #[cfg(test)]
    pub fn test_stream<T: Send + 'static + Debug + From<char>>(generations: usize) -> Stream<T> {
        Stream {
            _pd: PhantomData,
            test_source: TestSource {
                generations_left: generations,
                rng: ChaChaRng::seed_from_u64(TestSource::SEED),
            },
        }
    }
}

mod cache;
use cache::MultiThreadedCache;
use tokio;
use tokio::sync::Semaphore;

use std::panic::AssertUnwindSafe;
use std::sync::Arc;

type Query = String;
type APIResult = String;
type Cache = MultiThreadedCache<Query, APIResult>;

pub async fn process_result<'a>(res: APIResult) {
    println!("Result found: {res}");
}

pub async fn do_stuff_here(mut stream: external::Stream<Query>) {
    // here we want to process the queries we get from the stream
    // and println! the response we get from the API
    // but to reduce the network traffic, we want to add a caching mechanism
    // to avoid making request for the same query over and over again
    //
    // Note, to prevent excessive inflight requests, we use a Semaphore to limit computations (backpressure)

    let semaphore: Arc<Semaphore> = Arc::new(Semaphore::new(10));

    let cache = Cache::new();

    let mut tasks = Vec::new();

    while let Some(query) = stream.next().await {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore should remain available");
        let cache = cache.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = permit;

            let Ok(v) = cache
                .get_or_compute(query, |query| AssertUnwindSafe(external::api_call(query)))
                .await;

            process_result((*v).clone()).await;
        }));
    }

    for task in tasks {
        task.await.expect("worker task should complete");
    }
}

#[cfg(test)]
mod tests {
    use super::{Query, do_stuff_here, external, process_result};
    use std::convert::Infallible;
    use std::future::Future;

    fn run_test<F: Future<Output = ()>>(future: F) {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("test runtime should build")
            .block_on(future);
    }

    #[test]
    fn api_call_works() {
        run_test(async {
            let result: Result<String, Infallible> = external::api_call("example".to_owned()).await;

            assert_eq!(result.expect("API call should succeed"), "example");
        });
    }

    #[test]
    fn process_result_works() {
        run_test(async {
            process_result("example".to_owned()).await;
        });
    }

    #[test]
    fn test_source_drives_process() {
        run_test(async {
            let stream = external::test_stream::<Query>(100);
            do_stuff_here(stream).await;
        });
    }
}
