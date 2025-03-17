use std::{collections::LinkedList, pin::Pin, sync::Arc};

use async_stream::stream;
use futures::{Stream, StreamExt};
use std::future::Future;

pub type OriginalData2Stream<O, F, FP> = fn(
    Arc<O>,
    FP,
) -> Pin<
    Box<
        dyn Future<Output = anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<F>> + Send>>>>
            + Send,
    >,
>;

const MAX_STORED_DATA: usize = 8;

/// This is used to shuffle the stream infinitely,
/// this is designed for the speed of shuffling, not the quality of shuffling.
/// And in exchange for speed, the quality of shuffling is reduced, and the memory usage is increased.
/// The best use case for this is when coming stream is large,
/// and the normal vector shuffle is too slow.
pub struct InfiniteShuffleStream<O, F, FP>
where
    O: Send + Sync + Unpin,
    F: Send + Sync + Unpin,
    FP: Send + Sync + Unpin + Clone,
{
    file_provider: FP,
    repeat: bool,
    /// A probability determine how messy the shuffle is, a higher value means less messy
    /// 1.0 means no shuffle, the default value is 0.3
    shuffule_probability: f64,
    /// up_probability = shuffule_probability + (1.0 - shuffule_probability) / 2
    /// stored for optimization
    up_probability: f64,
    original_data: Arc<O>,
    original_data_2_stream: OriginalData2Stream<O, F, FP>,
}

impl<O, F, FP> InfiniteShuffleStream<O, F, FP>
where
    O: Send + Sync + Unpin,
    F: Send + Sync + Unpin,
    FP: Send + Sync + Unpin + Clone,
{
    pub fn new(
        file_provider: FP,
        original_data: Arc<O>,
        repeat: bool,
        shuffle: bool,
        original_data_2_stream: OriginalData2Stream<O, F, FP>,
    ) -> Self {
        let shuffule_probability = if shuffle { 0.3 } else { 1.0 };
        let up_probability = shuffule_probability + (1.0 - shuffule_probability) / 2.0;

        Self {
            file_provider,
            repeat,
            shuffule_probability,
            up_probability,
            original_data,
            original_data_2_stream,
        }
    }

    /// return a stream that will shuffle the data infinitely
    pub fn stream(&'_ self) -> impl Stream<Item = anyhow::Result<F>> + Send + '_ {
        let s = stream! {
            let mut stored_data = LinkedList::new();
            'outer: loop {
                let mut stream = (self.original_data_2_stream)(self.original_data.clone(), self.file_provider.clone()).await?;
                'inner: loop {
                    match stream.next().await {
                        Some(Ok(data)) => {
                            // rand a value between 0.0 and 1.0
                            let rand_value = rand::random::<f64>();
                            let data = if rand_value < self.shuffule_probability {
                                yield Ok(data);
                                continue;
                            } else if rand_value < self.up_probability {
                                stored_data.push_front(data);
                                if stored_data.len() > MAX_STORED_DATA {
                                    stored_data.pop_back()
                                } else {
                                    None
                                }
                            } else {
                                stored_data.push_back(data);
                                if stored_data.len() > MAX_STORED_DATA {
                                    stored_data.pop_front()
                                } else {
                                    None
                                }
                            };

                            if let Some(data) = data {
                                yield Ok(data);
                            }
                        }
                        Some(Err(err)) => {
                            yield Err(err);
                        }
                        None => {
                            if self.repeat {
                                break 'inner;
                            } else if stored_data.is_empty() {
                                break 'outer;
                            } else {
                                // consume all data in stored_data
                                let rand_value = rand::random::<f64>();
                                if rand_value < 0.5 {
                                    yield Ok(stored_data.pop_front().unwrap());
                                } else {
                                    yield Ok(stored_data.pop_back().unwrap());
                                }
                            }
                        }
                    }
                }
            }
        };

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::pin_mut;
    use std::collections::HashSet;

    #[derive(Clone)]
    struct MockFileProvider;

    struct MockOriginalData;

    fn mock_data_stream(
        _original_data: Arc<MockOriginalData>,
        _fp: MockFileProvider,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = anyhow::Result<
                        Pin<Box<dyn Stream<Item = anyhow::Result<i32>> + Send>>,
                    >,
                > + Send,
        >,
    > {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let stream: Pin<Box<dyn Stream<Item = anyhow::Result<i32>> + Send>> =
            Box::pin(futures::stream::iter(values.into_iter().map(Ok)));
        let stream = async move || Ok(stream);
        let stream = (stream)();
        Box::pin(stream)
    }

    #[tokio::test]
    async fn test_stream_no_repeat_no_shuffle() {
        let original_data = Arc::new(MockOriginalData);
        let file_provider = MockFileProvider;

        let shuffle_stream = InfiniteShuffleStream::new(
            file_provider,
            original_data,
            false, // no repeat
            false, // no shuffle
            mock_data_stream as OriginalData2Stream<MockOriginalData, i32, MockFileProvider>,
        );

        let stream = shuffle_stream.stream();
        pin_mut!(stream);

        let mut results = Vec::new();

        while let Some(item) = stream.next().await {
            results.push(item.unwrap());
        }

        // When no shuffle is enabled, we expect the items in their original order
        // Since shuffle probability is set to 1.0
        assert_eq!(results, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[tokio::test]
    async fn test_stream_with_repeat() {
        let original_data = Arc::new(MockOriginalData);
        let file_provider = MockFileProvider;

        let shuffle_stream = InfiniteShuffleStream::new(
            file_provider,
            original_data,
            true,  // repeat
            false, // no shuffle
            mock_data_stream,
        );

        let stream = shuffle_stream.stream();
        pin_mut!(stream);

        let mut results = Vec::new();

        // With repeat enabled, the stream will continue indefinitely
        // So we only take the first 20 items
        for _ in 0..20 {
            if let Some(item) = stream.next().await {
                results.push(item.unwrap());
            }
        }

        // We should get the sequence repeated
        assert_eq!(results[0..10], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(results[10..20], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[tokio::test]
    async fn test_stream_with_shuffle() {
        let original_data = Arc::new(MockOriginalData);
        let file_provider = MockFileProvider;

        let shuffle_stream = InfiniteShuffleStream::new(
            file_provider,
            original_data,
            false, // no repeat
            true,  // shuffle
            mock_data_stream,
        );

        let stream = shuffle_stream.stream();
        pin_mut!(stream);

        let mut results = Vec::new();

        while let Some(item) = stream.next().await {
            results.push(item.unwrap());
        }

        // With shuffle enabled, we expect:
        // 1. All original items should still be present
        // 2. The order should likely be different from the original

        // Check if all original items are present
        let result_set: HashSet<_> = results.iter().collect();
        assert_eq!(result_set.len(), 10);
        for i in 1..=10 {
            assert!(result_set.contains(&i));
        }

        // It's possible but unlikely that the shuffle produces the original order
        // This is a weak test but better than nothing
        let original_order = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert!(
            results.len() == original_order.len(),
            "Expected same number of items"
        );
    }
}
