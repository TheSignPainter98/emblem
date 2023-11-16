use crate::{
    log::{Logger, Verbosity},
    Log, Result,
};

pub struct BatchLogger {
    verbosity: Verbosity,
    logs: Vec<Log>,
}

impl BatchLogger {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            logs: vec![],
        }
    }

    pub fn logs(&self) -> &[Log] {
        &self.logs
    }
}

impl Logger for BatchLogger {
    fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn print(&mut self, log: Log) -> Result<()> {
        self.logs.push(log);
        Ok(())
    }

    fn report(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::log::AnnotationType;
    use strum::IntoEnumIterator;

    #[test]
    fn logs() {
        for verbosity in Verbosity::iter() {
            let mut logger = BatchLogger::new(verbosity);
            assert_eq!(verbosity, logger.verbosity());

            let logs = [
                Log::new(AnnotationType::Error, "hello"),
                Log::new(AnnotationType::Warning, "world"),
            ];
            for log in logs.iter() {
                logger.print(log.clone()).unwrap();
            }
            assert_eq!(logs, logger.logs());
        }
    }
}
