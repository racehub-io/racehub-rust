use anyhow::{Result, bail};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_PACKET: usize = 65507;
const MAX_COMPRESSED: usize = 1024 * 1024;

pub struct RawArchive {
    directory: PathBuf,
    simulator: String,
    files: BTreeMap<String, (String, BufWriter<File>, u64)>,
}
impl RawArchive {
    pub fn new(directory: &Path, simulator: &str) -> Self {
        Self {
            directory: directory.into(),
            simulator: simulator.into(),
            files: BTreeMap::new(),
        }
    }
    pub fn append(&mut self, timestamp: u64, bytes: &[u8]) -> Result<()> {
        let uid = (self.simulator == "f1" && bytes.len() >= 29 && bytes[0..3] == [232, 7, 24])
            .then(|| u64::from_le_bytes(bytes[7..15].try_into().unwrap()))
            .filter(|uid| *uid != 0);
        let key = uid
            .map(|uid| uid.to_string())
            .unwrap_or_else(|| "raw".into());
        if !self.files.contains_key(&key) {
            if self.files.len() >= 256 {
                bail!("The raw archive exceeds 256 source sessions.");
            }
            let name = format!("original-{key}-{}.rhc", uuid::Uuid::new_v4());
            let mut file = BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(self.directory.join(&name))?,
            );
            if let Some(uid) = uid {
                file.write_all(b"F1TEL\x01")?;
                file.write_u64::<BigEndian>(uid)?;
            } else {
                file.write_all(b"RHRAW\x01")?;
                file.write_u8(match self.simulator.as_str() {
                    "f1" => 0,
                    "iracing" => 1,
                    _ => 2,
                })?;
            }
            file.write_u64::<BigEndian>(timestamp / 1_000_000)?;
            file.write_all(&[0; 32])?;
            self.files.insert(key.clone(), (name, file, 0));
        }
        let (_, file, count) = self.files.get_mut(&key).unwrap();
        let compressed = zstd::encode_all(bytes, 3)?;
        file.write_u64::<BigEndian>(timestamp)?;
        file.write_u8(if uid.is_some() { bytes[6] } else { 255 })?;
        file.write_u32::<BigEndian>(compressed.len() as u32)?;
        file.write_all(&compressed)?;
        *count += 1;
        if *count % 100 == 0 {
            file.flush()?;
        }
        Ok(())
    }
    pub fn finish(mut self) -> Result<Vec<Value>> {
        let mut entries = Vec::new();
        for (name, file, count) in self.files.values_mut() {
            file.flush()?;
            file.get_ref().sync_all()?;
            entries
                .push(json!({"name":name,"size":file.get_ref().metadata()?.len(),"packets":count}));
        }
        Ok(entries)
    }
}

pub struct RawReader {
    file: BufReader<File>,
    pub simulator: String,
}
impl RawReader {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut magic = [0; 5];
        file.read_exact(&mut magic)?;
        if file.read_u8()? != 1 {
            bail!("Unsupported raw archive version.");
        }
        let simulator = match &magic {
            b"F1TEL" => {
                file.read_u64::<BigEndian>()?;
                "f1"
            }
            b"RHRAW" => match file.read_u8()? {
                0 => "f1",
                1 => "iracing",
                2 => "rfactor2",
                _ => bail!("Unknown raw archive simulator."),
            },
            _ => bail!("Unknown raw archive format."),
        }
        .to_string();
        file.read_u64::<BigEndian>()?;
        let mut reserved = [0; 32];
        file.read_exact(&mut reserved)?;
        Ok(Self { file, simulator })
    }
    pub fn next(&mut self) -> Result<Option<(u64, Vec<u8>)>> {
        let mut stamp = [0; 8];
        if self.file.read(&mut stamp[..1])? == 0 {
            return Ok(None);
        }
        self.file.read_exact(&mut stamp[1..])?;
        self.file.read_u8()?;
        let length = self.file.read_u32::<BigEndian>()? as usize;
        if length == 0 || length > MAX_COMPRESSED {
            bail!("Invalid compressed packet length.");
        }
        let mut compressed = vec![0; length];
        self.file.read_exact(&mut compressed)?;
        let mut bytes = Vec::new();
        zstd::stream::read::Decoder::new(compressed.as_slice())?
            .take((MAX_PACKET + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_PACKET {
            bail!("Invalid expanded packet length.");
        }
        Ok(Some((u64::from_be_bytes(stamp), bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_packets_round_trip_through_rhc_and_reject_truncated_frames() -> Result<()> {
        let directory = std::env::temp_dir().join(format!("racehub-raw-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory)?;
        let record: Value = serde_json::from_str(
            include_str!("../../capture/tests/recordings/2024.json")
                .lines()
                .nth(7)
                .unwrap(),
        )?;
        let packet = record["message"]["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as u8)
            .collect::<Vec<_>>();
        let stamp = chrono::DateTime::parse_from_rfc3339(record["time"].as_str().unwrap())?
            .timestamp_micros() as u64;
        let mut archive = RawArchive::new(&directory, "f1");
        archive.append(stamp, &packet)?;
        archive.append(stamp, &packet)?;
        let entries = archive.finish()?;
        let path = directory.join(entries[0]["name"].as_str().unwrap());
        let mut reader = RawReader::open(&path)?;
        assert_eq!(reader.simulator, "f1");
        assert_eq!(reader.next()?.unwrap(), (stamp, packet.clone()));
        assert_eq!(reader.next()?.unwrap(), (stamp, packet.clone()));
        assert!(reader.next()?.is_none());
        drop(reader);
        let mut legacy = capture::TelemetryReader::open(&path)?;
        assert_eq!(legacy.read_next_packet()?.unwrap().data, packet);
        drop(legacy);
        let file = OpenOptions::new().write(true).open(&path)?;
        file.set_len(file.metadata()?.len() - 3)?;
        drop(file);
        let mut reader = RawReader::open(&path)?;
        assert!(reader.next()?.is_some());
        assert!(reader.next().is_err());
        drop(reader);
        std::fs::remove_dir_all(directory)?;
        Ok(())
    }
}
