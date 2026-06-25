use crate::models::{CommandError, ProcessSnapshot};
use crate::providers::ProcessProvider;

pub struct ProcessManager<TProvider>
where
    TProvider: ProcessProvider,
{
    provider: TProvider,
}

impl<TProvider> ProcessManager<TProvider>
where
    TProvider: ProcessProvider,
{
    pub fn new(provider: TProvider) -> Self {
        Self { provider }
    }

    pub fn snapshot(&self) -> Result<ProcessSnapshot, CommandError> {
        self.provider.snapshot()
    }
}
