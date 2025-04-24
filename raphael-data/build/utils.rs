use std::{collections::VecDeque, io::BufRead, path::Path};

#[track_caller]
pub fn read_csv_data<RecordType>(path: impl AsRef<Path>) -> impl Iterator<Item = RecordType>
where
    RecordType: serde::de::DeserializeOwned,
{
    let reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    let lines: Vec<_> = reader
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match index {
            0 | 2 => None, // skip 1st and 3rd lines
            _ => Some(line.unwrap()),
        })
        .collect();
    let bytes: VecDeque<u8> = lines.join("\n").into_bytes().into();
    let reader = csv::Reader::from_reader(bytes);
    reader.into_deserialize::<RecordType>().map(|r| r.unwrap())
}
