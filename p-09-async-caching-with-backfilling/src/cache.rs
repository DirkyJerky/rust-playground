use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::panic::{self, RefUnwindSafe, UnwindSafe};
use std::sync::{Arc, Mutex};

use futures_util::FutureExt;

use computation_handles::{FollowerComputationHandle, LeaderComputationHandle};

mod computation_handles {
    //! Handles for passing computation results around across threads.
    //!
    //! This module relies on tokio's `sync::watch` channels for sharing computation results.
    //!
    //! When representing a computation result with a channel,
    //! The following invariants are expected to be upheld
    //! - A channel is initially None, to signify no results yet
    //! - A channel may be updated at most once, with the results of the computation
    //! - A channel may be dropped, to signal that the computation either
    //!   - errored
    //!   - was cancelled, by way of its owning Future being dropped
    //!   - or panicked.
    //!
    //! The types used in this module will enforce these invariants.

    use std::sync::Arc;
    use tokio::sync::watch::{Receiver, Sender, channel};

    /// A result of a computation, as seen in a `watch` channel
    pub type ComputationResult<V> = Option<Arc<V>>;

    /// The main computation's link to the `watch` channel.
    ///
    /// The single-use invariant will be tracked
    /// by `.take()`ing the inner Some option for operations that send
    ///
    /// If this channel isn't used to send a result after a computation finishes, (error, panic, etc),
    /// then the unique Sender will `drop` and close the channel.
    pub struct LeaderComputationHandle<V> {
        sender: Option<Sender<ComputationResult<V>>>,
    }
    /// A computation that can be listened to by other threads
    pub struct FollowerComputationHandle<V> {
        reciever: Receiver<ComputationResult<V>>,
    }

    impl<V> Clone for FollowerComputationHandle<V> {
        fn clone(&self) -> Self {
            FollowerComputationHandle {
                reciever: self.reciever.clone(),
            }
        }
    }

    impl<V> LeaderComputationHandle<V> {
        pub fn new() -> (Self, FollowerComputationHandle<V>) {
            let (sender, reciever) = channel::<ComputationResult<V>>(None);

            (
                LeaderComputationHandle {
                    sender: Some(sender),
                },
                FollowerComputationHandle { reciever },
            )
        }

        /// Send a computation result, consuming this computation handle
        //
        // This send should always succeed
        //
        // The situations where a value hypothetically may not send would be the following
        // - The send channel has closed (no more recievers)
        //   - If no one is listening, thats okay, we don't need to concern ourselves with it.
        //   - The cache invariant of holding entry references for de-duplication consistency should prevent this as well.
        // - The send channel has already been used and consumed
        //   - This should never happen due to this type's core invariant.
        pub fn send(mut self, result: Arc<V>) -> () {
            if let Some(sender) = self.sender.take() {
                let _ = sender.send(Some(result));
            } else {
                // critical!("Sender should be available to use")
            }
        }
    }

    impl<V> FollowerComputationHandle<V> {
        // Wait for the computation to finish.
        // After this wait succeeds, the value is expected to never update again.
        async fn wait_inner(&mut self) {
            if let Err(_) = self.reciever.changed().await {
                // critical!("Sender was dropped without sending None");
            }
        }

        fn get(&mut self) -> ComputationResult<V> {
            let v = self.reciever.borrow_and_update();
            let res: ComputationResult<V> = v.clone();

            res
        }

        /// Waits for the value, consuming this handle
        /// Once this function returns, the inner channel is never expected to update again.
        pub async fn wait(mut self) -> ComputationResult<V> {
            self.wait_inner().await;
            self.get()
        }
    }
}

/// A single cache entry
enum Entry<V> {
    /// The completed value for this key.
    // Wrapped in Arc so that the completed value can be referenced in multiple locations,
    // including for transmission using the InProgress reciever below
    Ready(Arc<V>),
    /// An in-progress computation.
    /// Cache hits that see this entry want to listen to the result of the computation.
    /// To see the computation result:
    /// - Clone the receiver
    /// - Drop their Mutex-guarded borrow of the hash map
    /// - Listen to the reciever to receive the computation result
    /// If the computation ultimately fails, then the reciever recieves the corresponding Error
    /// If the computation ultimately succeeds, then the reciever receieves the value
    ///
    /// The original inner Reciever
    /// - Should *never* be modified.
    /// - Should be cloned to interact with the inner value.
    /// - Will be discarded once the computation finishes.
    ///
    /// The return value in the receiver is initially None
    /// but is expected to update exactly once with
    /// - None if the computation canceled
    /// - Some(Ok(Arc<...>)) if the computation succeeded
    /// - Some(Err(Arc<...>)) if the computation errored
    InProgress(FollowerComputationHandle<V>),
}

/// A multi-threaded K-V cache, for use by the query runner.
///
/// This cache will de-duplicate value computations.
//
// The following invariants are expected to be upheld
// - Entries will only be modified by callers in possesion of the Mutex guard
// - Entries will always be an accurate representation of the corresponding computation to whomever hold the guard
// - Accesses to the hashmap should be brief and only be held while control flow is within this module
//   - This ensures that the HashMap will never get poisoned, unless its by our hand.
#[derive(Clone)]
pub struct MultiThreadedCache<K, V> {
    cache: Arc<Mutex<HashMap<K, Entry<V>>>>,
}

/// The three possible roles a cache user can take on
enum CallerRole<V> {
    /// If the value is already ready and can be returned immediately
    Ready(Arc<V>),
    /// If the value isn't ready, should be computed, and the computation needs to occur
    ShouldCompute(LeaderComputationHandle<V>),
    /// If the value isn't ready, should be computed, and another process is performing the computation
    ShouldWait(FollowerComputationHandle<V>),
}

impl<K, V> MultiThreadedCache<K, V> {
    pub fn new() -> Self {
        MultiThreadedCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Attempt
    pub async fn get_or_compute<F, Fut, E>(&self, key: K, compute: F) -> Result<Arc<V>, E>
    where
        F: Fn(K) -> Fut + RefUnwindSafe,
        Fut: Future<Output = Result<V, E>> + UnwindSafe,
        K: Eq + Hash + Clone + UnwindSafe,
    {
        loop {
            let role: CallerRole<V> = {
                // Get the lock for the hashmap
                // If the mutex is poisoned, then *our* code panicked to cause it.
                // As such, this error should be propagated.
                let mut map = self.cache.lock().expect("Lock should be healthy.");

                if let Some(entry) = map.get(&key) {
                    match entry {
                        Entry::Ready(v) => CallerRole::Ready(v.clone()),
                        Entry::InProgress(follower) => CallerRole::ShouldWait(follower.clone()),
                    }
                } else {
                    let (leader, follower) = LeaderComputationHandle::new();
                    // Key needs to be `Clone` here to allow for waiting cache users
                    // to retry the computation and insert their own in progress values
                    // TODO: Rework the lifetimes here, the key consumption at this point
                    // should be the final use of the key, so Clone might not be necessary.
                    map.insert(key.clone(), Entry::InProgress(follower));
                    CallerRole::ShouldCompute(leader)
                }
            };

            match role {
                // If we're ready, return it
                CallerRole::Ready(v) => return Ok(v),
                // If we need to compute, do that
                CallerRole::ShouldCompute(leader) => {
                    let res = compute(key.clone()).catch_unwind().await;

                    match res {
                        Ok(Ok(value)) => {
                            let value = Arc::new(value);

                            {
                                let mut map = self.cache.lock().expect("Lock should be healthy.");
                                map.insert(key.clone(), Entry::Ready(value.clone()));
                            }

                            leader.send(value.clone());
                            return Ok(value);
                        }
                        Ok(Err(error)) => {
                            {
                                let mut map = self.cache.lock().expect("Lock should be healthy.");
                                map.remove(&key);
                            }

                            return Err(error)
                        },
                        Err(panic) => {
                            {
                                let mut map = self.cache.lock().expect("Lock should be healthy.");
                                map.remove(&key);
                            }
                            panic::resume_unwind(panic)
                        }
                    }
                }
                // If we need to wait, wait and see what the result is
                CallerRole::ShouldWait(follower) => {
                    if let Some(res) = follower.wait().await {
                        return Ok(res);
                    } else {
                        // At this point, the result will have failed, and the original entry will have been modified
                        // Continue back to the top of the loop to try agian.
                        // This follower may become a leader computation, or listen to a different leader.
                        continue;
                    }
                }
            }
        }
    }
}
