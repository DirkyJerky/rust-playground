mod external {

    /// performs a call to a third party API
    pub async fn api_call(query_param: &str) -> String {
        unimplemented!();
    }

    use core::fmt::Debug;
    use core::marker::PhantomData;

    pub struct Stream<T: Send + 'static + Debug> {
        _pd: PhantomData<T>,
    }

    impl<T: Send + 'static + Debug> Stream<T> {
        /// returns the next element of the stream
        pub async fn next(&mut self) -> Option<T> {
            unimplemented!();
        }
    }
}

type Query = String;

pub async fn do_stuff_here(mut stream: external::Stream<Query>) {
    // here we want to process the queries we get from the stream
    // and println! the response we get from the API
    // but to reduce the network traffic, we want to add a caching mechanism
    // to avoid making request for the same query over and over again
}
