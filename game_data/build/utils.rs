use std::path::Path;

pub fn read_csv_data<RecordType>(path: impl AsRef<Path>) -> impl Iterator<Item = RecordType>
where
    RecordType: serde::de::DeserializeOwned,
{
    let reader = csv::Reader::from_path(path).unwrap();
    reader.into_deserialize::<RecordType>().map(|r| r.unwrap())
}
