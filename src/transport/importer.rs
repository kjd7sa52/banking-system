use crate::transport::record::Record;
use std::path::PathBuf;

pub struct CvsFileImporter {
    reader: csv::Reader<std::fs::File>,
}

impl CvsFileImporter {
    pub fn new(transactions: PathBuf) -> Result<Self, csv::Error> {
        let reader = csv::ReaderBuilder::new()
            .quoting(false)
            .trim(csv::Trim::All)
            .from_path(transactions)?;
        Ok(Self { reader })
    }

    pub fn read_rows(&mut self) -> csv::DeserializeRecordsIter<'_, std::fs::File, Record> {
        self.reader.deserialize::<Record>()
    }
}
