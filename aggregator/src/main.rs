mod raw;
mod storage;

use anyhow::{Context, Result, bail};
use capture::{native::Decoder, protocols::TelemetryRow};
use clap::{Args, Parser, Subcommand, ValueEnum};
use fs2::FileExt;
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use storage::ColumnarSession;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader as AsyncReader},
    net::UdpSocket,
    sync::mpsc,
};

#[derive(Parser)]
#[command(about = "Local native telemetry capture and Parquet conversion")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Capture(Capture),
    Convert(Convert),
    Benchmark(Benchmark),
}
#[derive(Clone, Debug, ValueEnum)]
enum Simulator {
    F1,
    Iracing,
    Rfactor2,
}
impl Simulator {
    fn name(&self) -> &'static str {
        match self {
            Self::F1 => "f1",
            Self::Iracing => "iracing",
            Self::Rfactor2 => "rfactor2",
        }
    }
    fn port(&self) -> u16 {
        match self {
            Self::F1 => 20777,
            Self::Iracing => 20900,
            Self::Rfactor2 => 20888,
        }
    }
}
#[derive(Clone, Debug, ValueEnum)]
enum Scope {
    Player,
    All,
}
impl Scope {
    fn name(&self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::All => "all",
        }
    }
}
#[derive(Args)]
struct Output {
    #[arg(long)]
    output_dir: PathBuf,
    #[arg(long)]
    recording_id: Option<uuid::Uuid>,
    #[arg(long, default_value = "Native recording")]
    name: String,
    #[arg(long, value_enum, default_value = "player")]
    scope: Scope,
}
#[derive(Args)]
struct Capture {
    #[arg(long, value_enum)]
    simulator: Simulator,
    #[arg(long, default_value = "127.0.0.1")]
    host: std::net::IpAddr,
    #[arg(long)]
    port: Option<u16>,
    #[command(flatten)]
    output: Output,
}
#[derive(Args)]
struct Convert {
    #[arg(long, value_enum)]
    simulator: Simulator,
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    source_session_id: Option<String>,
    #[command(flatten)]
    output: Output,
}
#[derive(Args)]
struct Benchmark {
    #[arg(long)]
    input: PathBuf,
    #[arg(long, default_value_t = 1000)]
    repeat: usize,
    #[arg(long)]
    canonical: bool,
    #[arg(long, default_value_t = 50)]
    warmup: usize,
}

fn emit(value: Value) {
    let mut output = std::io::stdout().lock();
    let _ = serde_json::to_writer(&mut output, &value);
    let _ = writeln!(output);
    let _ = output.flush();
}
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}
fn prepare(output: &Output) -> Result<(File, String)> {
    if output.name.len() > 256 {
        bail!("The recording name exceeds 256 bytes.");
    }
    fs::create_dir_all(&output.output_dir)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(output.output_dir.join(".capture.lock"))?;
    lock.try_lock_exclusive()
        .context("Another native capture uses this directory.")?;
    if fs::read_dir(&output.output_dir)?
        .filter_map(std::result::Result::ok)
        .any(|entry| entry.file_name() != ".capture.lock")
    {
        bail!("The output directory contains recording files. Select a new directory.");
    }
    Ok((
        lock,
        output
            .recording_id
            .unwrap_or_else(uuid::Uuid::new_v4)
            .to_string(),
    ))
}

#[derive(Default)]
struct Counters {
    received: AtomicU64,
    rejected: AtomicU64,
    dropped: AtomicU64,
    rows: AtomicU64,
    error: Mutex<Option<String>>,
    preview: Mutex<Vec<TelemetryRow>>,
}
impl Counters {
    fn status(&self) -> Value {
        json!({"event":"status","received":self.received.load(Ordering::Relaxed),"rejected":self.rejected.load(Ordering::Relaxed),"dropped":self.dropped.load(Ordering::Relaxed),"rows":self.rows.load(Ordering::Relaxed),"error":self.error.lock().unwrap().clone(),"captureRate":"source","queueCapacity":1024})
    }
    fn error(&self, error: impl ToString) {
        *self.error.lock().unwrap() = Some(error.to_string());
    }
}

struct Recording {
    decoder: Decoder,
    storage: ColumnarSession,
    raw: RawArchive,
    scope: Scope,
    session_key: Option<String>,
    source_clock: Option<f64>,
    source_filter: Option<String>,
    counters: Arc<Counters>,
}
use raw::RawArchive;
impl Recording {
    fn new(
        simulator: &str,
        output: &Output,
        id: &str,
        counters: Arc<Counters>,
        source_filter: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            decoder: Decoder::new(simulator).map_err(anyhow::Error::msg)?,
            storage: ColumnarSession::new(
                output.output_dir.clone(),
                id,
                simulator,
                &output.name,
                output.scope.name(),
            )?,
            raw: RawArchive::new(&output.output_dir, simulator),
            scope: output.scope.clone(),
            session_key: None,
            source_clock: None,
            source_filter,
            counters,
        })
    }
    fn packet(&mut self, time: u64, bytes: &[u8]) -> Result<()> {
        self.raw.append(time, bytes)?;
        if matches!(&self.decoder, Decoder::F1(_))
            && bytes.len() >= 29
            && bytes[..3] == [232, 7, 24]
            && self.source_filter.as_ref().is_some_and(|filter| {
                *filter != u64::from_le_bytes(bytes[7..15].try_into().unwrap()).to_string()
            })
        {
            return Ok(());
        }
        let packet = match self.decoder.decode(bytes) {
            Ok(packet) => packet,
            Err(error) => {
                self.counters.rejected.fetch_add(1, Ordering::Relaxed);
                self.counters.error(error);
                return Ok(());
            }
        };
        if let Some(filter) = &self.source_filter
            && packet.session_key.as_ref() != Some(filter)
        {
            return Ok(());
        }
        if let Some(key) = &packet.session_key {
            if self
                .session_key
                .as_ref()
                .is_some_and(|previous| previous != key)
            {
                bail!(
                    "The simulator started another source session. The recording stopped to keep sessions separate."
                );
            }
            self.session_key = Some(key.clone());
            self.storage.metadata["source_session_id"] = json!(key);
        }
        let clock = if matches!(&self.decoder, Decoder::F1(_)) {
            Some(f32::from_le_bytes(bytes[15..19].try_into().unwrap()) as f64)
        } else {
            packet
                .rows
                .iter()
                .filter_map(|row| row.get("sessionTime").and_then(Value::as_f64))
                .reduce(f64::max)
        };
        if let Some(clock) = clock {
            if self
                .source_clock
                .is_some_and(|previous| clock + 2.0 < previous)
            {
                bail!(
                    "The source clock moved backwards by more than two seconds. Start a new recording after a flashback or restart."
                );
            }
            self.source_clock = Some(
                self.source_clock
                    .map_or(clock, |previous| previous.max(clock)),
            );
        }
        if matches!(self.scope, Scope::Player) && packet.player_participant_id.is_none() {
            self.counters.error(
                "Player identity is not available. Select all scope for a spectator recording.",
            );
            return Ok(());
        }
        let mut preview = Vec::new();
        for row in packet.rows {
            if matches!(self.scope, Scope::Player)
                && row.get("participantId").and_then(Value::as_str)
                    != packet.player_participant_id.as_deref()
            {
                continue;
            }
            self.storage.append(&row)?;
            self.counters.rows.fetch_add(1, Ordering::Relaxed);
            if preview.len() < 22 {
                preview.push(row);
            }
        }
        if !preview.is_empty() {
            *self.counters.preview.lock().unwrap() = preview;
        }
        Ok(())
    }
    fn finish(mut self, error: Option<&str>) -> Result<Value> {
        self.storage.metadata["rawFiles"] = json!(self.raw.finish()?);
        self.storage.metadata["received"] = json!(self.counters.received.load(Ordering::Relaxed));
        self.storage.metadata["rejected"] = json!(self.counters.rejected.load(Ordering::Relaxed));
        self.storage.metadata["dropped"] = json!(self.counters.dropped.load(Ordering::Relaxed));
        let empty_error = (self.storage.rows == 0
            && self.counters.received.load(Ordering::Relaxed) > 0)
            .then(|| {
                self.counters
                    .error
                    .lock()
                    .unwrap()
                    .clone()
                    .unwrap_or_else(|| {
                        "No telemetry rows were decoded. The original packets were saved.".into()
                    })
            });
        self.storage.finish(error.or(empty_error.as_deref()))
    }
}

async fn capture(options: Capture) -> Result<()> {
    let (_lock, id) = prepare(&options.output)?;
    let simulator = options.simulator.name();
    let port = options.port.unwrap_or_else(|| options.simulator.port());
    let socket = UdpSocket::bind((options.host, port))
        .await
        .context("Cannot bind the telemetry UDP port.")?;
    let counters = Arc::new(Counters::default());
    let worker_counters = counters.clone();
    let mut recording = Recording::new(simulator, &options.output, &id, counters.clone(), None)?;
    let (sender, mut receiver) = mpsc::channel::<(u64, Vec<u8>)>(1024);
    #[cfg(unix)]
    let mut termination =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let terminate = async move {
        #[cfg(unix)]
        {
            termination.recv().await;
        }
        #[cfg(not(unix))]
        std::future::pending::<()>().await;
    };
    tokio::pin!(terminate);
    let mut worker = tokio::task::spawn_blocking(move || -> Result<Value> {
        let mut error = None;
        while let Some((time, bytes)) = receiver.blocking_recv() {
            if error.is_some() {
                recording.raw.append(time, &bytes)?;
                continue;
            }
            if let Err(cause) = recording.packet(time, &bytes) {
                error = Some(cause.to_string());
                worker_counters.error(&cause);
                receiver.close();
            }
        }
        recording.finish(error.as_deref())
    });
    emit(
        json!({"event":"ready","port":socket.local_addr()?.port(),"id":id,"simulator":simulator,"scope":options.output.scope.name(),"captureRate":"source","queueCapacity":1024,"rowGroupRows":storage::GROUP_ROWS}),
    );
    let mut bytes = vec![0; raw::MAX_PACKET + 1];
    let mut input = AsyncReader::new(tokio::io::stdin()).take(4097);
    let mut line = String::new();
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    let mut result = None;
    let mut queue_error: Option<String> = None;
    loop {
        tokio::select! {
            packet=socket.recv_from(&mut bytes)=>{
                let (length,_)=match packet {Ok(packet)=>packet,Err(error)=>{queue_error=Some(format!("Cannot receive a UDP packet: {error}"));break;}};
                counters.received.fetch_add(1,Ordering::Relaxed);
                if length>raw::MAX_PACKET {counters.rejected.fetch_add(1,Ordering::Relaxed);continue;}
                if let Err(error)=sender.try_send((timestamp(),bytes[..length].to_vec())) {
                    counters.dropped.fetch_add(1,Ordering::Relaxed);
                    queue_error=Some(match error {
                        mpsc::error::TrySendError::Full(_)=>"The native packet queue is full. Capture stopped. The source rate exceeded disk or decoder throughput.".into(),
                        mpsc::error::TrySendError::Closed(_)=>counters.error.lock().unwrap().clone().unwrap_or_else(||"The native writer stopped before it could accept a packet.".into()),
                    });
                    break;
                }
            }
            command=input.read_line(&mut line)=>{
                let count=match command {Ok(count)=>count,Err(error)=>{queue_error=Some(format!("Cannot read the stop command: {error}"));break;}};
                if count==0 || line.trim()=="stop" {break;}
                if line.len()>4096 {queue_error=Some("The capture command exceeds 4096 bytes.".into());break;}
                line.clear();
            }
            _=tokio::signal::ctrl_c()=>break,
            _= &mut terminate=>break,
            completed=&mut worker=>{result=Some(completed??);break;}
            _=interval.tick()=>{emit(counters.status());let rows=std::mem::take(&mut *counters.preview.lock().unwrap());if !rows.is_empty(){emit(json!({"event":"sample","rows":rows}));}}
        }
    }
    if let Some(error) = &queue_error {
        counters.error(error);
    }
    drop(socket);
    drop(sender);
    let mut session = match result {
        Some(value) => value,
        None => worker.await??,
    };
    if let Some(error) = queue_error {
        session["error"] = json!(&error);
        let path = options.output.output_dir.join("session.json");
        let mut metadata: Value = serde_json::from_reader(File::open(&path)?)?;
        metadata["error"] = json!(&error);
        atomic_json(&path, &metadata)?;
    }
    emit(counters.status());
    emit(json!({"event":"complete","session":session,"manifest":"session.json"}));
    Ok(())
}
fn atomic_json(path: &Path, value: &Value) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    serde_json::to_writer(&mut file, value)?;
    file.flush()?;
    file.sync_all()?;
    drop(file);
    fs::rename(temp, path)?;
    #[cfg(unix)]
    File::open(path.parent().unwrap())?.sync_all()?;
    Ok(())
}

fn fixture_packets(path: &Path, mut packet: impl FnMut(u64, &[u8]) -> Result<()>) -> Result<()> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut line = Vec::new();
    loop {
        line.clear();
        let count = (&mut reader)
            .take(2 * 1024 * 1024 + 1)
            .read_until(b'\n', &mut line)?;
        if count == 0 {
            break;
        }
        if line.len() > 2 * 1024 * 1024 {
            bail!("A recording line exceeds 2 MiB.");
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let record: Value = serde_json::from_slice(&line)?;
        let bytes = record["message"]["data"]
            .as_array()
            .context("The input is not a raw packet recording.")?;
        if bytes.is_empty() || bytes.len() > raw::MAX_PACKET {
            bail!("Invalid source packet length.");
        }
        let bytes = bytes
            .iter()
            .map(|value| {
                value
                    .as_u64()
                    .filter(|v| *v <= 255)
                    .map(|v| v as u8)
                    .context("Invalid source packet byte.")
            })
            .collect::<Result<Vec<_>>>()?;
        let time = record["time"]
            .as_str()
            .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
            .and_then(|time| u64::try_from(time.timestamp_micros()).ok())
            .or_else(|| record["time"].as_u64())
            .context("The source packet has no valid receive time.")?;
        packet(time, &bytes)?;
    }
    Ok(())
}
fn convert(options: Convert) -> Result<()> {
    let (_lock, id) = prepare(&options.output)?;
    let counters = Arc::new(Counters::default());
    let mut recording = Recording::new(
        options.simulator.name(),
        &options.output,
        &id,
        counters.clone(),
        options.source_session_id,
    )?;
    let mut magic = [0; 5];
    File::open(&options.input)?.read_exact(&mut magic)?;
    let mut consume = |time: u64, bytes: &[u8]| {
        counters.received.fetch_add(1, Ordering::Relaxed);
        recording.packet(time, bytes)
    };
    let result = (|| -> Result<()> {
        if &magic == b"F1TEL" || &magic == b"RHRAW" {
            let mut reader = raw::RawReader::open(&options.input)?;
            if reader.simulator != options.simulator.name() {
                bail!("The raw archive simulator does not match --simulator.");
            }
            let mut result = Ok(());
            while let Some((time, bytes)) = reader.next()? {
                if let Err(error) = consume(time, &bytes) {
                    result = Err(error);
                    break;
                }
            }
            result
        } else {
            fixture_packets(&options.input, &mut consume)
        }
    })();
    let error = result.err().map(|error| error.to_string());
    let session = recording.finish(error.as_deref())?;
    emit(json!({"event":"complete","session":session,"manifest":"session.json"}));
    if let Some(error) = error {
        bail!(error);
    }
    Ok(())
}
fn benchmark(options: Benchmark) -> Result<()> {
    if options.repeat == 0 || options.repeat > 1_000_000 || options.warmup > 10000 {
        bail!("Select a repeat count from 1 to 1000000.");
    }
    let mut packets = Vec::new();
    fixture_packets(&options.input, |_, bytes| {
        if packets.len() >= 100000 {
            bail!("The benchmark input exceeds 100000 packets.");
        }
        packets.push(bytes.to_vec());
        Ok(())
    })?;
    use capture::TelemetryPacket;
    let mut decoder = Decoder::new("f1").map_err(anyhow::Error::msg)?;
    let mut start = Instant::now();
    let mut parsed = 0u64;
    let mut rejected = 0u64;
    for round in 0..options.repeat + options.warmup {
        if round == options.warmup {
            start = Instant::now();
            parsed = 0;
            rejected = 0;
        }
        for bytes in &packets {
            if options.canonical {
                match decoder.decode(std::hint::black_box(bytes)) {
                    Ok(value) => {
                        std::hint::black_box(value.rows);
                        parsed += 1
                    }
                    Err(_) => rejected += 1,
                }
            } else {
                match capture::F1TelemetryPacket::from_bytes(std::hint::black_box(bytes)) {
                    Ok(value) => {
                        std::hint::black_box(value);
                        parsed += 1
                    }
                    Err(_) => rejected += 1,
                }
            }
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    emit(
        json!({"event":"benchmark","mode":if options.canonical {"native parser and canonical rows"}else{"direct native parser"},"sourcePackets":packets.len(),"repeat":options.repeat,"warmup":options.warmup,"replayedPackets":packets.len()*options.repeat,"parsed":parsed,"rejected":rejected,"seconds":elapsed,"packetsPerSecond":parsed as f64/elapsed,"note":"Repeated licensed source packets. This is a parser benchmark, not a live capture."}),
    );
    Ok(())
}
fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Cannot start the capture runtime.");
    let result = runtime.block_on(async {
        match Cli::parse().command {
            Command::Capture(options) => capture(options).await,
            Command::Convert(options) => convert(options),
            Command::Benchmark(options) => benchmark(options),
        }
    });
    runtime.shutdown_timeout(Duration::from_millis(100));
    if let Err(error) = result {
        emit(json!({"event":"error","error":format!("{error:#}")}));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn records() -> Vec<Value> {
        include_str!("../../capture/tests/recordings/2024.json")
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    fn bytes(record: &Value) -> Vec<u8> {
        record["message"]["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as u8)
            .collect()
    }
    fn output() -> Output {
        Output {
            output_dir: std::env::temp_dir()
                .join(format!("racehub-native-{}", uuid::Uuid::new_v4())),
            recording_id: None,
            name: "Actual source packet test".into(),
            scope: Scope::Player,
        }
    }
    fn feed(recording: &mut Recording, record: &Value) -> Result<()> {
        let time = chrono::DateTime::parse_from_rfc3339(record["time"].as_str().unwrap())?
            .timestamp_micros() as u64;
        recording.packet(time, &bytes(record))
    }
    #[test]
    fn excluded_source_uid_does_not_clear_selected_lap_state() -> Result<()> {
        let output = output();
        let (_lock, id) = prepare(&output)?;
        let counters = Arc::new(Counters::default());
        let records = records();
        let selected = "15747386989728148181";
        let lap = records
            .iter()
            .find(|record| {
                record["packetID"] == "lapData"
                    && record["parsed"]["m_header"]["m_sessionUID"] == selected
            })
            .unwrap();
        let excluded = records
            .iter()
            .find(|record| {
                record["packetID"] == "lapData"
                    && record["parsed"]["m_header"]["m_sessionUID"] != selected
            })
            .unwrap();
        let status = records
            .iter()
            .find(|record| record["packetID"] == "carStatus")
            .unwrap();
        let mut recording =
            Recording::new("f1", &output, &id, counters.clone(), Some(selected.into()))?;
        feed(&mut recording, lap)?;
        feed(&mut recording, excluded)?;
        feed(&mut recording, status)?;
        let preview = counters.preview.lock().unwrap();
        assert_eq!(preview[0]["packetId"], 7);
        assert_eq!(
            preview[0]["lap_source_time"],
            lap["parsed"]["m_header"]["m_sessionTime"]
        );
        assert!(preview[0].contains_key("lap_number"));
        drop(preview);
        recording.finish(None)?;
        drop(_lock);
        fs::remove_dir_all(output.output_dir)?;
        Ok(())
    }
    #[test]
    fn actual_clock_rewind_stops_before_append_but_small_reordering_is_allowed() -> Result<()> {
        let output = output();
        let (_lock, id) = prepare(&output)?;
        let counters = Arc::new(Counters::default());
        let records = records();
        let mut recording = Recording::new("f1", &output, &id, counters.clone(), None)?;
        let damage = records
            .iter()
            .find(|record| record["packetID"] == "carDamage")
            .unwrap();
        let motion = records
            .iter()
            .find(|record| record["packetID"] == "motion")
            .unwrap();
        let telemetry = records
            .iter()
            .find(|record| record["packetID"] == "carTelemetry")
            .unwrap();
        feed(&mut recording, damage)?;
        feed(&mut recording, motion)?;
        let count = recording.storage.rows;
        let error = feed(&mut recording, telemetry).unwrap_err().to_string();
        assert!(error.contains("backwards"));
        assert_eq!(count, recording.storage.rows);
        recording.finish(Some(&error))?;
        let manifest: Value =
            serde_json::from_reader(File::open(output.output_dir.join("session.json"))?)?;
        assert_eq!(manifest["rowCount"], count);
        assert!(manifest["error"].as_str().unwrap().contains("backwards"));
        assert_eq!(manifest["rawFiles"][0]["packets"], 3);
        drop(_lock);
        fs::remove_dir_all(output.output_dir)?;
        Ok(())
    }
    #[test]
    fn output_lock_rejects_a_second_writer_and_releases_on_drop() -> Result<()> {
        let output = output();
        let (lock, _) = prepare(&output)?;
        assert!(prepare(&output).is_err());
        drop(lock);
        let (_lock, _) = prepare(&output)?;
        drop(_lock);
        fs::remove_dir_all(output.output_dir)?;
        Ok(())
    }
    #[test]
    fn truncated_raw_conversion_keeps_completed_source_rows() -> Result<()> {
        let source = output();
        let (source_lock, id) = prepare(&source)?;
        let records = records();
        let mut archive = RawArchive::new(&source.output_dir, "f1");
        for name in ["lapData", "carStatus", "carDamage"] {
            let record = records
                .iter()
                .find(|record| {
                    record["packetID"] == name
                        && record["parsed"]["m_header"]["m_sessionUID"] == "15747386989728148181"
                })
                .unwrap();
            let time = chrono::DateTime::parse_from_rfc3339(record["time"].as_str().unwrap())?
                .timestamp_micros() as u64;
            archive.append(time, &bytes(record))?;
        }
        let entries = archive.finish()?;
        let path = source.output_dir.join(entries[0]["name"].as_str().unwrap());
        let file = OpenOptions::new().write(true).open(&path)?;
        file.set_len(file.metadata()?.len() - 3)?;
        drop(file);
        let target = output();
        let directory = target.output_dir.clone();
        let error = convert(Convert {
            simulator: Simulator::F1,
            input: path,
            source_session_id: None,
            output: target,
        })
        .unwrap_err();
        assert!(error.to_string().contains("fill whole buffer"));
        let metadata: Value = serde_json::from_reader(File::open(directory.join("session.json"))?)?;
        assert_eq!(metadata["rowCount"], 2);
        assert_eq!(metadata["state"], "complete");
        assert!(metadata["error"].is_string());
        drop(source_lock);
        let _ = id;
        fs::remove_dir_all(source.output_dir)?;
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
