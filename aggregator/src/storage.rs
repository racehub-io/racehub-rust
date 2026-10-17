use anyhow::{Result, bail};
use arrow_array::{
    ArrayRef, RecordBatch,
    builder::{BooleanBuilder, Float64Builder, StringBuilder},
};
use arrow_ord::sort::{SortColumn, lexsort_to_indices};
use arrow_schema::{DataType, Field, Schema, SortOptions};
use capture::protocols::TelemetryRow;
use parquet::{
    arrow::ArrowWriter,
    basic::{Compression, ZstdLevel},
    file::{metadata::KeyValue, properties::WriterProperties},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const GROUP_ROWS: usize = 8192;
const PART_ROWS: usize = 65536;

enum Column {
    Number(Float64Builder),
    Text(StringBuilder),
    Flag(BooleanBuilder),
}
impl Column {
    fn new(value: &Value) -> Result<Self> {
        Ok(match value {
            Value::Number(_) => Self::Number(Float64Builder::new()),
            Value::String(_) => Self::Text(StringBuilder::new()),
            Value::Bool(_) => Self::Flag(BooleanBuilder::new()),
            _ => bail!("The row contains an unsupported channel type."),
        })
    }
    fn accepts(&self, value: &Value) -> bool {
        value.is_null()
            || matches!(
                (self, value),
                (Self::Number(_), Value::Number(_))
                    | (Self::Text(_), Value::String(_))
                    | (Self::Flag(_), Value::Bool(_))
            )
    }
    fn append(&mut self, value: Option<&Value>) -> Result<()> {
        match self {
            Self::Number(builder) => {
                builder.append_option(value.and_then(Value::as_f64).filter(|n| n.is_finite()))
            }
            Self::Text(builder) => builder.append_option(value.and_then(Value::as_str)),
            Self::Flag(builder) => builder.append_option(value.and_then(Value::as_bool)),
        }
        Ok(())
    }
    fn datatype(&self) -> DataType {
        match self {
            Self::Number(_) => DataType::Float64,
            Self::Text(_) => DataType::Utf8,
            Self::Flag(_) => DataType::Boolean,
        }
    }
    fn finish(&mut self) -> ArrayRef {
        match self {
            Self::Number(v) => Arc::new(v.finish()),
            Self::Text(v) => Arc::new(v.finish()),
            Self::Flag(v) => Arc::new(v.finish()),
        }
    }
}

pub struct ColumnarSession {
    pub metadata: Value,
    directory: PathBuf,
    columns: BTreeMap<String, Column>,
    buffered: usize,
    buffered_bytes: usize,
    writer: Option<ArrowWriter<File>>,
    part_rows: usize,
    part_number: usize,
    pub rows: u64,
    pub parts: Vec<Value>,
}

fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    Ok(())
}

impl ColumnarSession {
    pub fn new(
        directory: PathBuf,
        id: &str,
        simulator: &str,
        name: &str,
        scope: &str,
    ) -> Result<Self> {
        let metadata = json!({"format":"racehub-telemetry","version":1,"id":id,"simulator":simulator,"name":name,
            "createdAt":chrono::Utc::now().to_rfc3339(),"trackName":"","scope":scope,"captureRate":"source","source_session_id":"","rowCount":0,"parts":[],"state":"recording"});
        let session = Self {
            metadata,
            directory,
            columns: BTreeMap::new(),
            buffered: 0,
            buffered_bytes: 0,
            writer: None,
            part_rows: 0,
            part_number: 0,
            rows: 0,
            parts: Vec::new(),
        };
        session.manifest(false, None)?;
        Ok(session)
    }

    pub fn append(&mut self, row: &TelemetryRow) -> Result<()> {
        if !row.get("participantId").is_some_and(Value::is_string)
            || !row
                .get("sessionTime")
                .and_then(Value::as_f64)
                .is_some_and(|time| time.is_finite() && time >= 0.0)
        {
            bail!("A native row has no valid participant or time.");
        }
        if row.len() > 512
            || row.iter().any(|(key, value)| {
                key.len() > 160 || value.as_str().is_some_and(|text| text.len() > 4096)
            })
        {
            bail!("A native row exceeds its channel limits.");
        }
        let schema_change = row
            .iter()
            .any(|(key, value)| !value.is_null() && !self.columns.contains_key(key));
        if schema_change && self.writer.is_some() {
            self.flush_group()?;
            self.finish_part()?;
        }
        if self.columns.len()
            + row
                .keys()
                .filter(|key| !self.columns.contains_key(*key))
                .count()
            > 512
        {
            bail!("The recording exceeds 512 channels.");
        }
        for (key, value) in row {
            if value.is_null() {
                continue;
            }
            if let Some(column) = self.columns.get(key) {
                if !column.accepts(value) {
                    bail!("The {key} channel changed type.");
                }
            } else {
                let mut column = Column::new(value)?;
                for _ in 0..self.buffered {
                    column.append(None)?;
                }
                self.columns.insert(key.clone(), column);
            }
        }
        let row_bytes = self.columns.len() * 9
            + row
                .values()
                .filter_map(Value::as_str)
                .map(str::len)
                .sum::<usize>();
        if self.buffered_bytes + row_bytes > 16 * 1024 * 1024 {
            self.flush_group()?;
        }
        for (key, column) in &mut self.columns {
            column.append(row.get(key))?;
        }
        self.buffered_bytes += row_bytes;
        for (source, target) in [("track_name", "trackName"), ("track_length", "trackLength")] {
            if let Some(value) = row.get(source) {
                self.metadata[target] = value.clone();
            }
        }
        if self.metadata["source_session_id"] == ""
            && let Some(value) = row.get("sessionId")
        {
            self.metadata["source_session_id"] = value.clone();
        }
        self.buffered += 1;
        self.rows += 1;
        if self.buffered >= GROUP_ROWS {
            self.flush_group()?;
        }
        Ok(())
    }

    fn flush_group(&mut self) -> Result<()> {
        if self.buffered == 0 {
            return Ok(());
        }
        let schema = Arc::new(Schema::new(
            self.columns
                .iter()
                .map(|(name, column)| Field::new(name, column.datatype(), true))
                .collect::<Vec<_>>(),
        ));
        let batch = RecordBatch::try_new(
            schema.clone(),
            self.columns.values_mut().map(Column::finish).collect(),
        )?;
        let indices = lexsort_to_indices(
            &[
                SortColumn {
                    values: batch.column_by_name("participantId").unwrap().clone(),
                    options: Some(SortOptions::default()),
                },
                SortColumn {
                    values: batch.column_by_name("sessionTime").unwrap().clone(),
                    options: Some(SortOptions::default()),
                },
            ],
            None,
        )?;
        let sorted = RecordBatch::try_new(
            schema.clone(),
            batch
                .columns()
                .iter()
                .map(|column| arrow_select::take::take(column.as_ref(), &indices, None))
                .collect::<std::result::Result<Vec<_>, _>>()?,
        )?;
        if self.writer.is_none() {
            self.part_number += 1;
            let file = OpenOptions::new().write(true).create_new(true).open(
                self.directory
                    .join(format!("part-{:06}.parquet.tmp", self.part_number)),
            )?;
            let properties = WriterProperties::builder()
                .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
                .set_max_row_group_row_count(Some(GROUP_ROWS))
                .build();
            self.writer = Some(ArrowWriter::try_new(file, schema, Some(properties))?);
        }
        let writer = self.writer.as_mut().unwrap();
        writer.write(&sorted)?;
        writer.flush()?;
        self.part_rows += self.buffered;
        self.buffered = 0;
        self.buffered_bytes = 0;
        if self.part_rows >= PART_ROWS {
            self.finish_part()?;
        }
        Ok(())
    }

    fn finish_part(&mut self) -> Result<()> {
        let Some(mut writer) = self.writer.take() else {
            return Ok(());
        };
        let mut metadata = self.metadata.clone();
        metadata.as_object_mut().unwrap().remove("parts");
        metadata["rowCount"] = json!(self.part_rows);
        writer.append_key_value_metadata(KeyValue::new(
            "racehub".into(),
            Some(serde_json::to_string(&metadata)?),
        ));
        let file = writer.into_inner()?;
        file.sync_all()?;
        drop(file);
        let name = format!("part-{:06}.parquet", self.part_number);
        let temporary = self.directory.join(format!("{name}.tmp"));
        let destination = self.directory.join(&name);
        if destination.exists() {
            bail!("A Parquet part already exists.");
        }
        fs::rename(temporary, &destination)?;
        sync_directory(&self.directory)?;
        self.parts.push(
            json!({"name":name,"size":fs::metadata(destination)?.len(),"rowCount":self.part_rows}),
        );
        self.part_rows = 0;
        self.manifest(false, None)?;
        Ok(())
    }

    pub fn manifest(&self, complete: bool, error: Option<&str>) -> Result<()> {
        let mut metadata = self.metadata.clone();
        metadata["rowCount"] = json!(
            self.parts
                .iter()
                .filter_map(|part| part["rowCount"].as_u64())
                .sum::<u64>()
        );
        metadata["parts"] = json!(self.parts);
        metadata["state"] = json!(if complete { "complete" } else { "recording" });
        if let Some(error) = error {
            metadata["error"] = json!(error);
        }
        let temporary = self
            .directory
            .join(format!("session-{}.tmp", uuid::Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        serde_json::to_writer(&mut file, &metadata)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        fs::rename(temporary, self.directory.join("session.json"))?;
        sync_directory(&self.directory)?;
        Ok(())
    }

    pub fn finish(mut self, error: Option<&str>) -> Result<Value> {
        self.flush_group()?;
        self.finish_part()?;
        self.manifest(true, error)?;
        self.metadata["state"] = json!("complete");
        self.metadata["rowCount"] = json!(self.rows);
        self.metadata["partCount"] = json!(self.parts.len());
        self.metadata.as_object_mut().unwrap().remove("parts");
        if let Some(error) = error {
            self.metadata["error"] = json!(error);
        }
        Ok(self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    #[test]
    fn repeated_actual_rows_use_multiple_row_groups_and_preserve_sparse_values() -> Result<()> {
        let directory =
            std::env::temp_dir().join(format!("racehub-parquet-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory)?;
        let record: Value = include_str!("../../capture/tests/recordings/2024.json")
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .find(|record| record["packetID"] == "carTelemetry")
            .unwrap();
        let bytes = record["message"]["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as u8)
            .collect::<Vec<_>>();
        let row = capture::native::F1Decoder::default()
            .decode(&bytes)
            .unwrap()
            .rows
            .remove(0);
        let mut session = ColumnarSession::new(
            directory.clone(),
            "actual-replay",
            "f1",
            "Repeated actual source rows",
            "player",
        )?;
        for _ in 0..73729 {
            session.append(&row)?;
        }
        session.finish(None)?;
        let manifest: Value = serde_json::from_reader(File::open(directory.join("session.json"))?)?;
        assert_eq!(manifest["rowCount"], 73729);
        assert_eq!(manifest["parts"].as_array().unwrap().len(), 2);
        let reader = ParquetRecordBatchReaderBuilder::try_new(File::open(
            directory.join("part-000001.parquet"),
        )?)?;
        assert_eq!(reader.metadata().num_row_groups(), 8);
        assert_eq!(reader.metadata().file_metadata().num_rows(), 65536);
        for group in reader.metadata().row_groups() {
            assert!(group.num_rows() <= GROUP_ROWS as i64);
        }
        let metadata: Value = serde_json::from_str(
            reader
                .metadata()
                .file_metadata()
                .key_value_metadata()
                .unwrap()
                .iter()
                .find(|entry| entry.key == "racehub")
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
        )?;
        assert_eq!(
            metadata["source_session_id"],
            record["parsed"]["m_header"]["m_sessionUID"]
        );
        let batch = reader.with_batch_size(10).build()?.next().unwrap()?;
        let rpm = batch
            .column_by_name("engine_rpm")
            .unwrap()
            .as_any()
            .downcast_ref::<arrow_array::Float64Array>()
            .unwrap();
        assert_eq!(rpm.value(0), row["engine_rpm"].as_f64().unwrap());
        assert!(batch.column_by_name("lap_number").is_none());
        assert!(batch.column_by_name("world_x").is_none());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
