use nyquest::AsyncClient;
use std::num::NonZeroU8;

pub struct Downloader {
    retries: usize,
    concurrent: usize,
    client: AsyncClient,
}

impl Downloader {
    async fn new() -> nyquest::Result<Self> {
        Ok(Self {
            retries: 3,
            concurrent: 4,
            client: nyquest::client::ClientBuilder::default()
                .build_async()
                .await?,
        })
    }
    fn retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
    fn concurrent(mut self, max: NonZeroU8) -> Self {
        self.concurrent = max.get() as usize;
        self
    }
}
