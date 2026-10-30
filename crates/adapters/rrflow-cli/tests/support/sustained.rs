//! Ordinary installed campaign. Explicit environment admission keeps a long
//! campaign out of ordinary correctness CI; failure artifacts are never green.
use super::Estate;
use rrd_client::RequestOptions;
use rrd_contract::{CloseSession, CreateSession, ReadStorageDiagnostics, SessionLimits};
use rrflow_eval::storage::*;
use std::{
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::task::JoinSet;
#[path = "sustained/cold.rs"]
mod cold;
#[path = "sustained/corpus.rs"]
mod corpus;
#[path = "sustained/damage.rs"]
mod damage;
#[path = "sustained/fleet.rs"]
mod fleet;
#[path = "sustained/lifecycle.rs"]
mod lifecycle;
#[path = "sustained/management.rs"]
mod management;
#[path = "sustained/native.rs"]
mod native;
#[path = "sustained/native_worker.rs"]
mod native_worker;
#[path = "sustained/reader.rs"]
mod reader;
#[path = "sustained/resource_window.rs"]
mod resource_window;
#[path = "sustained/sample_task.rs"]
mod sample_task;
#[path = "sustained/sample_trace.rs"]
mod sample_trace;
#[path = "sustained/workload.rs"]
mod workload;
use workload::{Prepared, Workload};

type DriverResult<T> = std::result::Result<T, String>;

fn retain_failed_lifecycle<T>(
    estate: &super::owned_directory::OwnedDirectory,
    result: DriverResult<T>,
) -> DriverResult<T> {
    if result.is_err() {
        eprintln!(
            "native_lifecycle_failure_estate={}",
            serde_json::json!({"path": estate.retain(), "private_state": true})
        );
    }
    result
}

#[test]
fn lifecycle_failure_retains_its_owned_estate_but_success_cleans_it() {
    let parent = tempfile::tempdir().unwrap();
    let success =
        super::owned_directory::OwnedDirectory::new(tempfile::tempdir_in(parent.path()).unwrap());
    let success_path = success.path().to_owned();
    assert_eq!(
        retain_failed_lifecycle(&success, Ok::<_, String>(7)).unwrap(),
        7
    );
    drop(success);
    assert!(!success_path.exists());

    let failed =
        super::owned_directory::OwnedDirectory::new(tempfile::tempdir_in(parent.path()).unwrap());
    let failed_path = failed.path().to_owned();
    std::fs::write(failed_path.join("sentinel"), b"retained").unwrap();
    assert_eq!(
        retain_failed_lifecycle::<()>(&failed, Err("fixture-failure".into())).unwrap_err(),
        "fixture-failure"
    );
    drop(failed);
    assert_eq!(
        std::fs::read(failed_path.join("sentinel")).unwrap(),
        b"retained"
    );
}

fn evidence_id(value: impl Into<String>) -> Id {
    value.into().try_into().unwrap()
}
fn ns(clock: Instant) -> u64 {
    clock
        .elapsed()
        .as_nanos()
        .try_into()
        .expect("bounded campaign clock")
}
fn bounded_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> DriverResult<T> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_DOCUMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decode(&bytes)
}

async fn session(
    estate: &Estate,
    client: rrd_client::RrdClient,
    seed: u64,
    campaign: &StorageCampaign,
    clients: u64,
    ordinal: u64,
) -> DriverResult<Workload> {
    let credential = rrd_engine::RrdEngine::read_installed_api_key(&estate.project)
        .map_err(|e| e.to_string())?;
    let name = format!("capacity-session-{ordinal}");
    let session = client
        .create_session(
            credential.principal_id.clone(),
            credential.credential(),
            CreateSession {
                limits: SessionLimits {
                    idle_timeout_ms: 600_000,
                    absolute_timeout_ms: 3_600_000,
                    max_open_transactions: clients
                        .try_into()
                        .map_err(|_| "client count overflow")?,
                },
            },
            RequestOptions::new(&name, &name, Some(&name), Some(u64::MAX)).unwrap(),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(Workload {
        native_worker: Default::default(),
        client,
        session,
        valid_at: estate.at,
        dimensions: campaign
            .allocation
            .vector_dimensions
            .get()
            .try_into()
            .map_err(|_| "vector dimensions overflow")?,
        seed,
        records: campaign.allocation.records.get(),
        read_distribution: campaign.read_distribution.clone(),
        edges_per_record: campaign
            .allocation
            .edges
            .get()
            .div_ceil(campaign.allocation.records.get())
            .try_into()
            .map_err(|_| "edge fanout overflow")?,
        vectors_per_record: campaign
            .allocation
            .vectors
            .get()
            .div_ceil(campaign.allocation.records.get())
            .try_into()
            .map_err(|_| "vector fanout overflow")?,
        payload_bytes: campaign
            .allocation
            .logical_source_bytes
            .get()
            .div_ceil(campaign.allocation.records.get())
            .max(1024)
            .try_into()
            .map_err(|_| "source payload overflow")?,
        query_budget: serde_json::from_value(estate.plan["configuration"]["query"].clone())
            .map_err(|e| e.to_string())?,
    })
}

/// Rotate only between bounded segments, after every in-flight request has
/// joined. Closing the old lease prevents a day of accumulated live sessions.
async fn rotate_session(
    estate: &Estate,
    work: &mut Workload,
    campaign: &StorageCampaign,
    clients: u64,
    ordinal: &mut u64,
    opened: &mut Instant,
) -> DriverResult<()> {
    if opened.elapsed() < Duration::from_secs(120) {
        return Ok(());
    }
    let key = format!("capacity-close-{ordinal}");
    work.client
        .close_session(
            &work.session,
            CloseSession {},
            RequestOptions::new(&key, &key, Some(&key), Some(u64::MAX)).unwrap(),
        )
        .await
        .map_err(|e| e.to_string())?;
    *ordinal = ordinal.checked_add(1).ok_or("session ordinal overflow")?;
    *work = session(
        estate,
        work.client.clone(),
        work.seed,
        campaign,
        clients,
        *ordinal,
    )
    .await?;
    *opened = Instant::now();
    Ok(())
}

async fn sample(
    workload: &Workload,
    pid: u32,
    clock: Instant,
    allocation: &Option<NativeAllocationReceipt>,
) -> DriverResult<ResourceSample> {
    sample_with_identity(workload, pid, clock, allocation)
        .await
        .map(|(sample, _, _)| sample)
}

#[derive(Clone, Copy)]
struct SampleTiming {
    total_ns: u64,
    host_ns: u64,
    rpc_ns: u64,
    memory_ns: u64,
    enforcement_ns: u64,
}

async fn sample_with_identity(
    workload: &Workload,
    pid: u32,
    clock: Instant,
    allocation: &Option<NativeAllocationReceipt>,
) -> DriverResult<(ResourceSample, String, SampleTiming)> {
    let started = Instant::now();
    let rpc_started = Instant::now();
    let request_id = sample_trace::next_request_id().map_err(str::to_string)?;
    let snapshot = workload
        .client
        .read_storage_diagnostics(
            &workload.session,
            ReadStorageDiagnostics {
                scope: format!("instance:{}", workload.client.instance_id()),
            },
            RequestOptions::read(&request_id, &request_id).map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;
    let rpc_ns = ns(rpc_started);
    let missing = |reason: &str| NativeMetric::Unavailable {
        reason: evidence_id(reason),
    };
    let native = workload.native_worker.acquire(pid, allocation).await?;
    // This is the parent-side completion time of the real native observation.
    let elapsed_ns = ns(clock);
    let host_ns = native.host_ns;
    let memory_ns = native.memory_ns;
    let enforcement_ns = native.enforcement_ns;
    let value = ResourceSample {
        elapsed_ns,
        pid,
        resident_bytes: NativeMetric::Observed {
            value: native.resident_bytes,
        },
        private_bytes: native.private_bytes,
        mapped_bytes: native.mapped_bytes,
        host_resident_bytes: NativeMetric::Observed {
            value: native.host_resident_bytes,
        },
        read_bytes: NativeMetric::Observed {
            value: native.read_bytes,
        },
        written_bytes: NativeMetric::Observed {
            value: native.written_bytes,
        },
        cpu_time_ns: NativeMetric::Observed {
            value: native.cpu_time_ns,
        },
        retained_bytes: match &snapshot.admission {
            Some(value) => NativeMetric::Observed {
                value: value
                    .retained_mutable_bytes
                    .checked_add(value.retained_projected_bytes)
                    .and_then(|bytes| bytes.checked_add(value.retained_source_bytes))
                    .and_then(|bytes| bytes.checked_add(value.native_transaction_bytes))
                    .ok_or("retained sum overflow")?,
            },
            None => missing("public-retained-gauge-unavailable"),
        },
        queue_requests: match &snapshot.admission {
            Some(value) => NativeMetric::Observed {
                value: value.outstanding_requests,
            },
            None => missing("public-queue-gauge-unavailable"),
        },
        maintenance_debt_segments: match &snapshot.rrflow_kv {
            Some(value) => NativeMetric::Observed {
                value: value.segment_set.compaction_debt_segments,
            },
            None => missing("public-debt-gauge-unavailable"),
        },
        allocation_enforcement: native.enforcement,
        allocation_receipt: allocation.clone(),
        storage: Some(snapshot),
    };
    let timing = SampleTiming {
        total_ns: ns(started),
        host_ns,
        rpc_ns,
        memory_ns,
        enforcement_ns,
    };
    sample_trace::record(sample_trace::Observation {
        request_id: &request_id,
        pid,
        elapsed_ns,
        total_ns: timing.total_ns,
        host_ns: timing.host_ns,
        rpc_ns: timing.rpc_ns,
        memory_ns: timing.memory_ns,
        enforcement_ns: timing.enforcement_ns,
    });
    Ok((value, request_id, timing))
}

struct ProgressContext<'a> {
    writer: &'a CampaignWriter,
    point: &'a OperatingPoint,
}

struct PointRun<'a> {
    campaign: &'a StorageCampaign,
    point: &'a OperatingPoint,
    workload: Workload,
    pid: u32,
    clock: Instant,
    allocation: Option<NativeAllocationReceipt>,
    generations: Vec<u64>,
    previous_resources: Vec<ResourceSample>,
    backup: management::ManagementTask<BackupEvidence>,
    sample_gate: Option<std::sync::Arc<sample_task::SampleGate>>,
}

impl PointRun<'_> {
    async fn sample(&self, writer: &CampaignWriter) -> DriverResult<ResourceSample> {
        let value = sample(&self.workload, self.pid, self.clock, &self.allocation).await?;
        Self::record_sample(writer, value)
    }
    fn record_sample(
        writer: &CampaignWriter,
        value: ResourceSample,
    ) -> DriverResult<ResourceSample> {
        writer.record_event(&ProgressEvent::Resource {
            segment: writer.next_index(),
            sample: Box::new(value.clone()),
        })?;
        Ok(value)
    }
    fn start_sample(&self, task: &mut sample_task::SampleTask<ResourceSample>) -> DriverResult<()> {
        self.start_sample_at(task, None)
    }
    fn start_sample_at(
        &self,
        task: &mut sample_task::SampleTask<ResourceSample>,
        not_before: Option<Instant>,
    ) -> DriverResult<()> {
        let work = self.workload.clone();
        let pid = self.pid;
        let clock = self.clock;
        let allocation = self.allocation.clone();
        let gate = self.sample_gate.clone();
        task.start(async move {
            if let Some(not_before) = not_before {
                tokio::time::sleep_until(not_before.into()).await;
            }
            if let Some(gate) = gate {
                gate.wait().await;
            }
            sample(&work, pid, clock, &allocation).await
        })
    }
    async fn drain(
        &self,
        writer: &CampaignWriter,
        duration_ms: u64,
    ) -> DriverResult<SustainedSegment> {
        if self.campaign.idle_segment_ms(duration_ms)? != duration_ms {
            return Err("idle duration exceeds its frozen segment admission".into());
        }
        let result = tokio::time::timeout(
            Duration::from_millis(self.campaign.storage.budgets.run_ms.get().min(180_000)),
            self.drain_inner(writer, duration_ms),
        )
        .await
        .map_err(|_| "idle segment exceeded its frozen deadline".to_owned())
        .and_then(std::convert::identity);
        let cleanup = self.workload.native_worker.shutdown().await;
        match (result, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
            (Err(error), Err(cleanup)) => Err(format!("{error}; native worker cleanup: {cleanup}")),
        }
    }

    async fn drain_inner(
        &self,
        writer: &CampaignWriter,
        duration_ms: u64,
    ) -> DriverResult<SustainedSegment> {
        let started = ns(self.clock);
        let baseline =
            self.workload
                .prepare("drain-baseline".into(), StorageLoadKind::PointRead, 0)?;
        let expected = ExpectedOperation {
            logical_id: evidence_id(&baseline.id),
            request_sha256: baseline.request_sha256.clone(),
            effect_sha256: baseline.effect_sha256.clone(),
            resolution: ExpectedResolution::Present,
        };
        let key = evidence_id(format!("drain-{}", writer.next_index()));
        let mut manifest = self.campaign.storage.clone();
        manifest.required_cases = vec![RequiredCase {
            id: key.clone(),
            operation: evidence_id("transaction-commit"),
            seed: self.workload.seed,
            expected_ledger_sha256: expected_digest(std::slice::from_ref(&expected))?,
        }];
        let mut resources = Vec::new();
        let deadline =
            self.clock + Duration::from_nanos(started) + Duration::from_millis(duration_ms);
        let period = Duration::from_millis(self.campaign.schedule.sample_ms.get());
        let mut next_sample = Instant::now();
        loop {
            resources.push(self.sample(writer).await?);
            if Instant::now() >= deadline {
                break;
            }
            if resources.len() as u64 >= self.campaign.evidence.samples_per_segment.get() {
                return Err("drain resource sample cap exhausted".into());
            }
            next_sample =
                resource_window::next_sample_deadline(next_sample, period, Instant::now())?;
            tokio::time::sleep_until(next_sample.min(deadline).into()).await;
        }
        writer.record_event(&ProgressEvent::LoadIntent {
            segment: writer.next_index(),
            point: self.point.id.clone(),
            seed: self.workload.seed,
            phase: LoadPhase::Drain,
            expected: expected.clone(),
            write: None,
        })?;
        writer.checkpoint_progress()?;
        let probe_started = ns(self.clock);
        let (receipt, probe_finished) = tokio::time::timeout(
            Duration::from_millis(self.campaign.schedule.operation_deadline_ms.get()),
            async {
                let receipt = self.workload.execute(&baseline).await?;
                let finished = ns(self.clock);
                self.workload.verify(&baseline).await?;
                Ok::<_, String>((receipt, finished))
            },
        )
        .await
        .map_err(|_| "drain probe deadline")??;
        let ledger = AcknowledgementLedger {
            expected: vec![expected],
            // This is the actual drain probe, not offered load or throughput.
            attempts: vec![Attempt {
                attempt_id: evidence_id(format!("drain-probe-{}", writer.next_index())),
                logical_id: evidence_id(&baseline.id),
                request_sha256: baseline.request_sha256.clone(),
                started_ns: probe_started - started,
                finished_ns: probe_finished - started,
                outcome: AttemptOutcome::Acknowledged {
                    receipt_sha256: receipt.receipt_sha256.clone(),
                },
            }],
            readback: vec![Readback::Present {
                logical_id: evidence_id(baseline.id),
                request_sha256: baseline.request_sha256,
                effect_sha256: baseline.effect_sha256.clone(),
                receipt_sha256: receipt.receipt_sha256,
                applications: 1u64.try_into().unwrap(),
            }],
        };
        writer.record_event(&ProgressEvent::Readback {
            segment: writer.next_index(),
            observation: ledger.readback[0].clone(),
        })?;
        writer.checkpoint_progress()?;
        let result = StorageResult {
            format_version: 1,
            manifest_sha256: manifest_digest(&manifest)?,
            cases: vec![CaseResult {
                id: key,
                seed: self.workload.seed,
                status: CaseStatus::Executed,
                faults: vec![],
                limitation: None,
                ledger,
            }],
        };
        Ok(SustainedSegment {
            format_version: 1,
            campaign_sha256: writer.identity().clone(),
            index: writer.next_index(),
            previous_sha256: writer.previous_sha256(),
            point: self.point.id.clone(),
            seed: self.workload.seed,
            phase: LoadPhase::Drain,
            started_ns: started,
            finished_ns: ns(self.clock),
            corpus: None,
            cold_start: None,
            reopens: None,
            backup_started_ns: None,
            backup: None,
            reader: None,
            manifest,
            result,
            attempts: vec![],
            resources,
            lanes: vec![],
        })
    }
    async fn segment(
        &mut self,
        writer: &CampaignWriter,
        phase: LoadPhase,
        origin: u64,
        first: u64,
        count: u64,
        final_segment: bool,
    ) -> DriverResult<SustainedSegment> {
        let campaign = self.campaign;
        let rate =
            self.point.requests_per_second.get() * if phase == LoadPhase::Burst { 2 } else { 1 };
        let scheduled =
            |sequence: u64| origin + (sequence as u128 * 1_000_000_000 / rate as u128) as u64;
        let started = if phase == LoadPhase::Drain {
            ns(self.clock)
        } else {
            scheduled(first)
        };
        let key = format!(
            "point-{}-seed-{}-segment-{}",
            self.point.id.as_str(),
            self.workload.seed,
            writer.next_index()
        );
        let mut prepared = Vec::new();
        let mut ledger = AcknowledgementLedger::default();
        // This complete expected ledger is frozen before any segment operation.
        for offset in 0..count {
            let sequence = first + offset;
            let name = format!("load-{}-{sequence}", writer.next_index());
            let kind = storage_load_kind(sequence);
            let operation = if kind == StorageLoadKind::MixedWrite {
                self.workload.prepare_load(
                    name,
                    WriteTransition {
                        slot: offset,
                        generation: self.generations[offset as usize],
                    },
                )?
            } else {
                self.workload.prepare(name, kind, sequence)?
            };
            ledger.expected.push(ExpectedOperation {
                logical_id: evidence_id(&operation.id),
                request_sha256: operation.request_sha256.clone(),
                effect_sha256: operation.effect_sha256.clone(),
                resolution: ExpectedResolution::Either,
            });
            prepared.push(operation);
        }
        for (expected, operation) in ledger.expected.iter().zip(&prepared) {
            writer.record_event(&ProgressEvent::LoadIntent {
                segment: writer.next_index(),
                point: self.point.id.clone(),
                seed: self.workload.seed,
                phase,
                expected: expected.clone(),
                write: operation.write,
            })?;
        }
        writer.checkpoint_progress()?;
        let mut manifest = campaign.storage.clone();
        manifest.required_cases = vec![RequiredCase {
            id: evidence_id(&key),
            operation: evidence_id("transaction-commit"),
            seed: self.workload.seed,
            expected_ledger_sha256: expected_digest(&ledger.expected)?,
        }];
        // Offered windows may overlap when the preceding segment drains late.
        // Its actual observations still cover that time; never invent a new
        // origin or lose their evidence merely because the ledger changed.
        let mut resources = resource_window::overlapping_samples(
            &self.previous_resources,
            started,
            self.pid,
            campaign.evidence.samples_per_segment.get() as usize,
        )?;
        for value in &resources {
            writer.record_event(&ProgressEvent::Resource {
                segment: writer.next_index(),
                sample: Box::new(value.clone()),
            })?;
        }
        let mut sample_task = sample_task::SampleTask::new();
        if phase == LoadPhase::Warmup && first == 0 {
            // The evaluator requires this real ownership baseline before any
            // initial warmup effect; later segment samples need no such fence.
            resources.push(self.sample(writer).await?);
        } else {
            let not_before = self
                .clock
                .checked_add(Duration::from_nanos(started))
                .ok_or("initial sample deadline overflow")?;
            self.start_sample_at(&mut sample_task, Some(not_before))?;
        }
        let held_reader = if phase == LoadPhase::Measured
            && campaign
                .required_lanes
                .contains(&CapacityLane::ReaderPinInterference)
            && prepared.iter().any(|operation| operation.write.is_some())
        {
            Some(
                reader::HeldReader::open(
                    &self.workload,
                    writer.next_index(),
                    self.clock,
                    campaign.reader_selection.as_ref(),
                )
                .await?,
            )
        } else {
            None
        };
        let mut attempts = Vec::new();
        let mut failure = ReadbackFailures::default();
        let mut joins = JoinSet::new();
        let mut backup_started_ns = None;
        let deadline = Duration::from_millis(campaign.schedule.operation_deadline_ms.get());
        let period = Duration::from_millis(campaign.schedule.sample_ms.get());
        let last_observed = self.clock
            + Duration::from_nanos(resources.last().map_or(started, |value| value.elapsed_ns));
        let mut next_sample =
            resource_window::next_sample_deadline(last_observed, period, Instant::now())?;
        for (offset, operation) in prepared.iter().enumerate() {
            let sequence = first + offset as u64;
            let due = self.clock + Duration::from_nanos(scheduled(sequence));
            loop {
                let sample_pending = sample_task.is_pending();
                tokio::select! {
                    biased;
                    result=joins.join_next(), if !joins.is_empty() => {
                        collect(result.ok_or("missing scheduled task")?.map_err(|_|"scheduled task panicked")?,started,&mut attempts,&mut ledger,&mut failure,writer)?;
                    }
                    _=tokio::time::sleep_until(due.into()) => break,
                    value=sample_task.finish(), if sample_pending => {
                        resources.push(Self::record_sample(writer, value?.ok_or("missing resource sample")?)?);
                        next_sample=resource_window::next_sample_deadline(next_sample,period,Instant::now())?;
                    }
                    _=tokio::time::sleep_until(next_sample.into()), if !sample_pending => {
                        if resources.len() as u64>=campaign.evidence.samples_per_segment.get() {return Err("resource sample cap exhausted".into());}
                        self.start_sample(&mut sample_task)?;
                    }
                }
            }
            let work = self.workload.clone();
            let operation = operation.clone();
            let clock = self.clock;
            if phase == LoadPhase::Measured && sequence == 14 {
                let work = work.clone();
                let name = format!("capacity-backup-{}", writer.next_index());
                let started_ns = ns(clock);
                backup_started_ns = Some(started_ns);
                writer.record_event(&ProgressEvent::BackupStarted {
                    segment: writer.next_index(),
                    point: self.point.id.clone(),
                    seed: self.workload.seed,
                    started_ns,
                })?;
                writer.checkpoint_progress()?;
                self.backup.start(
                    clock + Duration::from_nanos(started_ns) + deadline,
                    lifecycle::backup_started(work, name, clock, started_ns),
                )?;
            }
            if joins.len() as u64 >= self.point.clients.get() {
                let at = ns(clock);
                collect(
                    (
                        sequence,
                        scheduled(sequence),
                        at,
                        at,
                        at,
                        operation,
                        Err("driver-concurrency-limit".into()),
                        None,
                        true,
                    ),
                    started,
                    &mut attempts,
                    &mut ledger,
                    &mut failure,
                    writer,
                )?;
            } else {
                let at = scheduled(sequence);
                joins.spawn(async move {
                    let began = ns(clock);
                    let expires = tokio::time::Instant::now() + deadline;
                    let result = tokio::time::timeout_at(expires, work.execute(&operation))
                        .await
                        .map_err(|_| "operation-deadline".to_owned())
                        .and_then(std::convert::identity);
                    // API latency ends at its actual response. Independent oracle
                    // work stays inside this same admitted client slot/deadline,
                    // so it cannot pause the next offered segment or grow tasks.
                    let finished = ns(clock);
                    let verification = if result.is_ok() {
                        Some(
                            tokio::time::timeout_at(expires, work.verify(&operation))
                                .await
                                .map_err(|_| "readback-deadline".to_owned())
                                .and_then(std::convert::identity),
                        )
                    } else {
                        None
                    };
                    (
                        sequence,
                        at,
                        began,
                        finished,
                        ns(clock),
                        operation,
                        result,
                        verification,
                        false,
                    )
                });
            }
        }
        while !joins.is_empty() {
            let sample_pending = sample_task.is_pending();
            tokio::select! {
                result=joins.join_next() => collect(result.ok_or("missing scheduled task")?.map_err(|_|"scheduled task panicked")?,started,&mut attempts,&mut ledger,&mut failure,writer)?,
                value=sample_task.finish(), if sample_pending => {
                    resources.push(Self::record_sample(writer, value?.ok_or("missing resource sample")?)?);
                    next_sample=resource_window::next_sample_deadline(next_sample,period,Instant::now())?;
                }
                _=tokio::time::sleep_until(next_sample.into()), if !sample_pending => {
                    if resources.len() as u64>=campaign.evidence.samples_per_segment.get(){return Err("resource sample cap exhausted".into());}
                    self.start_sample(&mut sample_task)?;
                }
            }
        }
        // All offers have been issued and joined. Complete the single owned
        // sample before the reader, backup or final resource boundary.
        if sample_task.is_pending() {
            let value = sample_task
                .finish()
                .await?
                .ok_or("missing resource sample")?;
            resources.push(Self::record_sample(writer, value)?);
            next_sample =
                resource_window::next_sample_deadline(next_sample, period, Instant::now())?;
        }
        let reader_receipt = match held_reader {
            Some(reader) => Some(reader.finish(&self.workload, self.clock).await?),
            None => None,
        };
        if let Some(observation) = &reader_receipt {
            writer.record_event(&ProgressEvent::ReaderInterference {
                segment: writer.next_index(),
                observation: Box::new(observation.clone()),
            })?;
            writer.checkpoint_progress()?;
        }
        // An evidence boundary must not join management work while more offers
        // remain. The task keeps its original absolute deadline across segments.
        let mut backup_receipt = self.backup.try_finish()?;
        while final_segment && self.backup.is_pending() {
            tokio::select! {
                value=self.backup.finish() => { backup_receipt=value?; }
                _=tokio::time::sleep_until(next_sample.into()) => {
                    if resources.len() as u64+1>=campaign.evidence.samples_per_segment.get() {
                        return Err("backup resource sample cap exhausted".into());
                    }
                    resources.push(self.sample(writer).await?);
                    next_sample=resource_window::next_sample_deadline(next_sample,period,Instant::now())?;
                }
            }
        }
        let verified = failure.count == 0;
        // Each working slot appears at most once in this admitted segment.
        // Only an independently verified response advances its next generation.
        for operation in &prepared {
            if ledger.readback.iter().any(|observation| {
                matches!(observation,
                Readback::Present { logical_id, .. } if logical_id.as_str() == operation.id)
            }) {
                if let Some(write) = operation.write {
                    self.generations[write.slot as usize] = write
                        .generation
                        .checked_add(1)
                        .filter(|n| *n <= MAX_EXACT_INTEGER)
                        .ok_or("working slot generation overflow")?;
                }
            }
        }
        let offered_end = self.clock + Duration::from_nanos(scheduled(first + count));
        while Instant::now() < offered_end {
            tokio::time::sleep_until(next_sample.min(offered_end).into()).await;
            if resources.len() as u64 + 1 >= campaign.evidence.samples_per_segment.get() {
                return Err("offered interval resource sample cap exhausted".into());
            }
            resources.push(self.sample(writer).await?);
            next_sample =
                resource_window::next_sample_deadline(next_sample, period, Instant::now())?;
        }
        resources.push(self.sample(writer).await?);
        if backup_receipt.is_none() {
            backup_receipt = self.backup.try_finish()?;
        }
        if let Some(observation) = &backup_receipt {
            writer.record_event(&ProgressEvent::Backup {
                segment: writer.next_index(),
                observation: observation.clone(),
            })?;
        }
        attempts.sort_by_key(|a| a.sequence);
        writer.checkpoint_progress()?;
        let mut lanes = match &backup_receipt {
            Some(raw) => vec![LaneEvidence {
                lane: CapacityLane::BackupUnderWrite,
                repetitions: 1u64.try_into()?,
                receipt_sha256: digest("c07g.backup-evidence", raw)?,
                acknowledged_readback_verified: verified,
            }],
            None => vec![],
        };
        if let Some(raw) = &reader_receipt {
            if raw.qualifies(&attempts) {
                lanes.push(LaneEvidence {
                    lane: CapacityLane::ReaderPinInterference,
                    repetitions: 1u64.try_into()?,
                    receipt_sha256: digest("c07g.reader-interference", raw)?,
                    acknowledged_readback_verified: verified,
                });
            }
        }
        let result = StorageResult {
            format_version: 1,
            manifest_sha256: manifest_digest(&manifest)?,
            cases: vec![CaseResult {
                id: evidence_id(key),
                seed: self.workload.seed,
                status: if verified {
                    CaseStatus::Executed
                } else {
                    CaseStatus::Failed
                },
                faults: failure.finish(),
                limitation: if verified {
                    None
                } else {
                    Some(evidence_id("independent-readback-failed"))
                },
                ledger,
            }],
        };
        if phase == LoadPhase::Measured && first == 0 {
            lanes.push(LaneEvidence {
                lane: CapacityLane::Warm,
                repetitions: 1u64.try_into()?,
                receipt_sha256: digest("c07g.warm-readback", &result)?,
                acknowledged_readback_verified: verified,
            });
        }
        // One admitted segment only. A failure to append/validate the segment
        // stops the campaign, so unvalidated observations cannot advance it.
        self.previous_resources.clone_from(&resources);
        Ok(SustainedSegment {
            format_version: 1,
            campaign_sha256: writer.identity().clone(),
            index: writer.next_index(),
            previous_sha256: writer.previous_sha256(),
            point: self.point.id.clone(),
            seed: self.workload.seed,
            phase,
            started_ns: started,
            finished_ns: ns(self.clock),
            corpus: None,
            cold_start: None,
            reopens: None,
            backup_started_ns,
            backup: backup_receipt,
            reader: reader_receipt,
            manifest,
            result,
            attempts,
            resources,
            lanes,
        })
    }
}

// One rolling failure digest bounds retained provenance independently of
// segment attempt count. It includes each failed logical identity and cause.
#[derive(Default)]
struct ReadbackFailures {
    count: u64,
    digest: Option<Digest>,
}
impl ReadbackFailures {
    fn record(&mut self, logical_id: &str, cause: &str) -> DriverResult<()> {
        self.count = self.count.checked_add(1).ok_or("failure count overflow")?;
        self.digest = Some(digest(
            "c07g.readback-failures",
            &(&self.digest, logical_id, cause),
        )?);
        Ok(())
    }
    fn finish(self) -> Vec<Failure> {
        self.digest.map_or_else(Vec::new, |artifact_sha256| {
            vec![Failure {
                phase: FailurePhase::Readback,
                code: evidence_id("independent-readback-failed"),
                occurrence: self.count.try_into().expect("nonempty failure history"),
                artifact_sha256,
            }]
        })
    }
}
fn failure_code(error: &str) -> Id {
    if error == "operation-deadline" {
        return evidence_id(error);
    }
    if let Some((code, detail)) = error.split_once(':') {
        if (code.starts_with("api-") || code.starts_with("client-"))
            && code.len() <= 80
            && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
            && detail.len() == 64
            && detail.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return evidence_id(code);
        }
    }
    evidence_id("request-or-oracle-failed")
}

#[test]
fn failed_campaign_preserves_closed_cause_and_readback_provenance() {
    assert_eq!(
        failure_code("operation-deadline").as_str(),
        "operation-deadline"
    );
    let sanitized = format!("api-conflict:{}", "a".repeat(64));
    assert_eq!(failure_code(&sanitized).as_str(), "api-conflict");
    assert_eq!(
        failure_code("api-secret:private detail").as_str(),
        "request-or-oracle-failed"
    );
    let manifest: StorageManifest = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../evaluation/rrflow-eval/tests/fixtures/storage/manifest.json"
    )))
    .unwrap();
    let mut result: StorageResult = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../evaluation/rrflow-eval/tests/fixtures/storage/result.json"
    )))
    .unwrap();
    result.manifest_sha256 = manifest_digest(&manifest).unwrap();
    result.cases[0].status = CaseStatus::Failed;
    result.cases[0].faults.clear();
    assert_eq!(
        validate_result(&manifest, &result).unwrap_err(),
        "failed case must retain failure provenance"
    );
    let mut failure = ReadbackFailures::default();
    failure.record("logical-a", &sanitized).unwrap();
    failure.record("logical-b", "readback differs").unwrap();
    result.cases[0].faults = failure.finish();
    assert_eq!(result.cases[0].faults[0].occurrence.get(), 2);
    let report = validate_result(&manifest, &result).unwrap();
    assert_eq!(report.failed, 1);
    assert!(!report.contract_passed);
}

type Completion = (
    u64,
    u64,
    u64,
    u64,
    u64,
    Prepared,
    DriverResult<workload::Completed>,
    Option<DriverResult<()>>,
    bool,
);
fn collect(
    (
        sequence,
        scheduled_ns,
        started_ns,
        finished_ns,
        verification_finished_ns,
        operation,
        result,
        verification,
        rejected,
    ): Completion,
    segment_start: u64,
    attempts: &mut Vec<LoadAttempt>,
    ledger: &mut AcknowledgementLedger,
    failure: &mut ReadbackFailures,
    writer: &CampaignWriter,
) -> DriverResult<()> {
    let (outcome, result_sha256, recorded) = match result {
        Ok(result) => (
            LoadOutcome::Acknowledged,
            result.receipt_sha256.clone(),
            AttemptOutcome::Acknowledged {
                receipt_sha256: result.receipt_sha256,
            },
        ),
        Err(_) if rejected => (
            LoadOutcome::Rejected,
            digest("c07g.saturated", &sequence)?,
            AttemptOutcome::Rejected {
                reason: evidence_id("driver-concurrency-limit"),
            },
        ),
        Err(error) if error.starts_with("rejected-") => (
            LoadOutcome::Rejected,
            digest("c07g.rejected-request", &(sequence, &error))?,
            AttemptOutcome::Rejected {
                reason: failure_code(error.strip_prefix("rejected-").expect("matched prefix")),
            },
        ),
        Err(error) => (
            LoadOutcome::Uncertain,
            digest("c07g.request-failure", &(sequence, &error))?,
            AttemptOutcome::Uncertain {
                reason: failure_code(&error),
            },
        ),
    };
    let readback = match outcome {
        LoadOutcome::Acknowledged => match verification {
            Some(Ok(())) => Some(Readback::Present {
                logical_id: evidence_id(&operation.id),
                request_sha256: operation.request_sha256.clone(),
                effect_sha256: operation.effect_sha256.clone(),
                receipt_sha256: result_sha256.clone(),
                applications: 1u64.try_into().unwrap(),
            }),
            Some(Err(error)) => {
                failure.record(&operation.id, &error)?;
                None
            }
            None => return Err("acknowledged worker omitted independent verification".into()),
        },
        LoadOutcome::Rejected => Some(Readback::Absent {
            logical_id: evidence_id(&operation.id),
        }),
        _ => {
            failure.record(&operation.id, result_sha256.as_str())?;
            None
        }
    };
    ledger.attempts.push(Attempt {
        attempt_id: evidence_id(format!("attempt-{sequence}")),
        logical_id: evidence_id(&operation.id),
        request_sha256: operation.request_sha256.clone(),
        started_ns: started_ns - segment_start,
        finished_ns: finished_ns - segment_start,
        outcome: recorded,
    });
    attempts.push(LoadAttempt {
        sequence,
        kind: operation.kind,
        write: operation.write,
        scheduled_ns,
        started_ns,
        finished_ns,
        verification_finished_ns: Some(verification_finished_ns),
        outcome,
        logical_id: evidence_id(operation.id),
        request_sha256: operation.request_sha256,
        result_sha256,
    });
    writer.record_event(&ProgressEvent::LoadOutcome {
        segment: writer.next_index(),
        attempt: attempts.last().ok_or("missing completed attempt")?.clone(),
    })?;
    if let Some(observation) = readback {
        writer.record_event(&ProgressEvent::Readback {
            segment: writer.next_index(),
            observation: observation.clone(),
        })?;
        ledger.readback.push(observation);
    }
    Ok(())
}

#[test]
#[ignore = "explicit frozen campaign and new evidence directory required; never substitutes for 24-hour evidence"]
fn installed_sustained_capacity() {
    let campaign_path = PathBuf::from(
        std::env::var_os("RRFLOW_SUSTAINED_CAMPAIGN").expect("explicit campaign path"),
    );
    let evidence =
        PathBuf::from(std::env::var_os("RRFLOW_SUSTAINED_EVIDENCE").expect("new evidence path"));
    let campaign: StorageCampaign = bounded_json(&campaign_path).unwrap();
    campaign.validate().unwrap();
    if campaign.allocation.estates.get() > 1 {
        return fleet::execute(campaign, evidence);
    }
    let mut writer = CampaignWriter::create(&evidence, campaign.clone()).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run(&campaign, &mut writer))
    }))
    .unwrap_or_else(|_| {
        Err("installed driver panicked; retained progress is incomplete evidence".into())
    });
    let (status, limitations) = if result.is_ok() {
        (CampaignStatus::Completed, vec![])
    } else {
        if let Err(error) = writer.record_event(&ProgressEvent::Failure {
            reason: evidence_id("installed-driver-failed"),
        }) {
            eprintln!("failure event could not be appended: {error}; sealing failed result");
        }
        (
            CampaignStatus::Failed,
            vec![evidence_id("installed-driver-failed")],
        )
    };
    let report = writer.finish(status, limitations).unwrap();
    println!(
        "native_sustained_result={}",
        serde_json::to_string(&report).unwrap()
    );
    assert!(
        result.is_ok(),
        "installed sustained driver failed: {result:?}"
    );
    assert!(
        campaign.purpose != CampaignPurpose::Qualification || report.qualification_passed,
        "campaign completed but did not satisfy its frozen qualification requirements"
    );
}

/// Short executable integration of the campaign's actual workload adapter.
/// It deliberately emits no NativeCapacity/soak qualification claim.
fn smoke_campaign() -> StorageCampaign {
    let mut storage: StorageManifest = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../evaluation/rrflow-eval/tests/fixtures/storage/manifest.json"
    )))
    .unwrap();
    storage.evidence_class = EvidenceClass::NativeCapacity;
    storage.budgets.estates = 1u64.try_into().unwrap();
    let limit = |value: u64| value.try_into().unwrap();
    StorageCampaign {
        format_version: 1,
        purpose: CampaignPurpose::Characterization,
        allocation: CapacityAllocation {
            owner_projection_sha256: digest("sample-smoke", &1).unwrap(),
            vcpus: limit(1),
            host_bytes: storage.budgets.host_bytes,
            engine_bytes: storage.budgets.process_bytes,
            estates: storage.budgets.estates,
            records: limit(8),
            edges: limit(24),
            vectors: limit(16),
            vector_dimensions: limit(16),
            logical_source_bytes: limit(8192),
        },
        schedule: SustainedSchedule {
            setup_ms: limit(1000),
            lifecycle_ms: limit(1000),
            warmup_ms: limit(2000),
            measured_ms: limit(2000),
            burst_ms: limit(2000),
            drain_ms: limit(2000),
            soak_ms: limit(2000),
            sample_ms: limit(100),
            operation_deadline_ms: limit(10_000),
            forced_reopens: limit(1),
        },
        read_distribution: ReadKeyDistribution::Uniform {},
        drain: DrainPolicy {
            maximum_mutable_growth_bytes: 0,
            maximum_projected_growth_bytes: 0,
            maximum_debt_segments: 0,
            stable_samples: limit(2),
        },
        seeds: vec![73],
        points: vec![OperatingPoint {
            id: evidence_id("sample-smoke"),
            clients: limit(4),
            requests_per_second: limit(5),
            maximum_error_basis_points: 10_000,
            maximum_p99_ns: limit(10_000_000_000),
        }],
        required_lanes: vec![CapacityLane::Warm],
        evidence: EvidenceLimits {
            segment_attempts: limit(6),
            segment_bytes: limit(MAX_DOCUMENT_BYTES),
            total_bytes: limit(64 * 1024 * 1024),
            segments: limit(10),
            samples_per_segment: limit(64),
        },
        reader_selection: None,
        storage,
    }
}

#[test]
#[ignore = "explicit installed trace correlation; not capacity qualification"]
fn installed_sampler_trace_matches_server_and_store() {
    let estate = Estate::install("fresh");
    let clock = Instant::now();
    let (mut child, client, _) = estate.start_for_with_trace(clock, Duration::from_secs(300), true);
    let credential = rrd_engine::RrdEngine::read_installed_api_key(&estate.project).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (request_id, mut work, before) = runtime.block_on(async {
        let lease = client
            .create_session(
                credential.principal_id.clone(),
                credential.credential(),
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 300_000,
                        absolute_timeout_ms: 600_000,
                        max_open_transactions: 4,
                    },
                },
                RequestOptions::mutation(
                    "trace-sample-session",
                    "trace-sample-session",
                    "trace-sample-session",
                )
                .unwrap(),
            )
            .await
            .unwrap();
        let work = Workload {
            native_worker: Default::default(),
            client,
            session: lease,
            valid_at: estate.at,
            dimensions: 16,
            seed: 73,
            records: 8,
            read_distribution: ReadKeyDistribution::Uniform {},
            edges_per_record: 3,
            vectors_per_record: 2,
            payload_bytes: 1024,
            query_budget: serde_json::from_value(estate.plan["configuration"]["query"].clone())
                .unwrap(),
        };
        work.initialize().await.unwrap();
        for ordinal in 0..work.records {
            let seed = work.seed_record(ordinal).unwrap();
            assert!(!work.execute(&seed).await.unwrap().idempotent_replay);
        }
        let before = work
            .verify_joined_search("seed-00000000000000000007", 1, 7, 5.53125)
            .await
            .unwrap();
        let (sample, request_id, _) = sample_with_identity(&work, child.id(), clock, &None)
            .await
            .unwrap();
        assert!(sample.storage.is_some());
        (request_id, work, before)
    });
    child.terminate();
    let output = child.finish().expect("reap owned traced server");
    check_sample_trace(&output.stderr, &request_id, credential.credential());
    let (mut restarted, client, _) =
        estate.start_for_with_trace(clock, Duration::from_secs(300), true);
    work.client = client;
    let second_id = runtime.block_on(async {
        let after = work
            .verify_joined_search("seed-00000000000000000007", 1, 7, 5.53125)
            .await
            .unwrap();
        assert_eq!(after.known_at_cursor, before.known_at_cursor);
        assert_eq!(after.hits, before.hits);
        let (sample, id, _) = sample_with_identity(&work, restarted.id(), clock, &None)
            .await
            .unwrap();
        assert!(sample.storage.is_some());
        id
    });
    assert_ne!(request_id, second_id);
    restarted.terminate();
    let output = restarted.finish().expect("reap restarted traced server");
    check_sample_trace(&output.stderr, &second_id, credential.credential());
}

fn check_sample_trace(bytes: &[u8], request_id: &str, credential: &str) -> [u64; 7] {
    let raw = std::str::from_utf8(bytes).unwrap();
    assert!(!raw.contains(request_id));
    assert!(!raw.contains(credential));
    let digest = rrd_core::digest::sha256_hex(request_id.as_bytes());
    let events: Vec<serde_json::Value> = raw
        .lines()
        .map(|line| serde_json::from_str(line).expect("trace is complete JSON"))
        .collect();
    let find = |target: &str| {
        let matching: Vec<_> = events
            .iter()
            .filter(|event| event["target"] == target && event["span"]["request_sha256"] == digest)
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "missing or duplicate correlated event: {target}"
        );
        matching[0]
    };
    let storage = find("rrd_store::diagnostic");
    let service = find("rrd_server::diagnostic");
    let field = |name: &str| storage["fields"][name].as_u64().unwrap();
    let wait = field("mutex_wait_ns");
    let assembly = field("mutex_held_assembly_ns");
    let profile = field("memtable_profile_ns");
    let elapsed = service["fields"]["service_elapsed_ns"].as_u64().unwrap();
    assert_eq!(service["fields"]["success"], true);
    assert!(profile <= assembly);
    assert!(u128::from(wait) + u128::from(assembly) <= u128::from(elapsed));
    let counts = [
        field("memtable_keys"),
        field("memtable_versions"),
        field("segment_count"),
    ];
    println!("installed sampler trace matched: request_sha256={digest} wait_ns={wait} assembly_ns={assembly} profile_ns={profile} service_ns={elapsed} keys={} versions={} segments={}", counts[0], counts[1], counts[2]);
    [
        wait, assembly, profile, elapsed, counts[0], counts[1], counts[2],
    ]
}

#[test]
#[ignore = "explicit bounded loaded diagnostic characterization; not capacity qualification"]
fn installed_sampler_trace_during_write_load() {
    const RECORDS: u64 = 4_000;
    const PERIOD: u64 = 125;
    let estate = Estate::install("fresh");
    let clock = Instant::now();
    let (mut child, client, _) = estate.start_for_with_trace(clock, Duration::from_secs(600), true);
    let credential = rrd_engine::RrdEngine::read_installed_api_key(&estate.project).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (mut work, observations, acknowledged, failures) = runtime.block_on(async {
        let session = client
            .create_session(
                credential.principal_id.clone(),
                credential.credential(),
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 300_000,
                        absolute_timeout_ms: 600_000,
                        max_open_transactions: 4,
                    },
                },
                RequestOptions::mutation("loaded-session", "loaded-session", "loaded-session")
                    .unwrap(),
            )
            .await
            .unwrap();
        let work = Workload {
            native_worker: Default::default(),
            client,
            session,
            valid_at: estate.at,
            dimensions: 16,
            seed: 73,
            records: RECORDS,
            read_distribution: ReadKeyDistribution::Uniform {},
            edges_per_record: 3,
            vectors_per_record: 2,
            payload_bytes: 1024,
            query_budget: serde_json::from_value(estate.plan["configuration"]["query"].clone())
                .unwrap(),
        };
        work.initialize().await.unwrap();
        let mut observations = Vec::new();
        let mut acknowledged = 0u64;
        let mut failures = Vec::new();
        let failed = |stage: &str, ordinal: u64, detail: &str| {
            format!(
                "{stage} ordinal={ordinal} error_sha256={}",
                rrd_core::digest::sha256_hex(detail.as_bytes())
            )
        };
        for ordinal in 0..RECORDS {
            let seed = work.seed_record(ordinal).unwrap();
            let written = if ordinal % PERIOD == 0 {
                let write = async {
                    let start = ns(clock);
                    let result = work.execute(&seed).await;
                    (start, ns(clock), result)
                };
                let diagnostic = async {
                    let start = ns(clock);
                    let result = sample_with_identity(&work, child.id(), clock, &None).await;
                    (start, ns(clock), result)
                };
                let ((write_start, write_end, written), (sample_start, sample_end, sampled)) =
                    tokio::join!(write, diagnostic);
                match sampled {
                    Ok((sample, id, timing)) if sample.storage.is_some() => {
                        observations.push((
                            id,
                            ordinal,
                            write_start,
                            write_end,
                            sample_start,
                            sample_end,
                            timing,
                        ));
                    }
                    Ok(_) => {
                        failures.push(failed("sample-missing-storage", ordinal, "missing storage"))
                    }
                    Err(error) => failures.push(failed("sample", ordinal, &error)),
                }
                written
            } else {
                work.execute(&seed).await
            };
            match written {
                Ok(result) if !result.idempotent_replay => acknowledged += 1,
                Ok(_) => failures.push(failed("unexpected-replay", ordinal, "replayed new seed")),
                Err(error) => failures.push(failed("write", ordinal, &error)),
            }
            if !failures.is_empty() {
                break;
            }
            if acknowledged.is_multiple_of(500) {
                println!(
                    "loaded_progress acknowledged={acknowledged} samples={}",
                    observations.len()
                );
            }
        }
        if failures.is_empty() {
            for ordinal in [0, 1_333, 2_666, RECORDS - 1] {
                if let Err(error) = work.verify(&work.seed_record(ordinal).unwrap()).await {
                    failures.push(failed("readback", ordinal, &error));
                    break;
                }
            }
        }
        (work, observations, acknowledged, failures)
    });
    let overlapping = observations
        .iter()
        .filter(|(_, _, ws, we, ss, se, _)| ws < se && ss < we)
        .count();
    child.terminate();
    let output = child.finish().expect("reap loaded traced server");
    for (id, ordinal, write_start, write_end, sample_start, sample_end, timing) in &observations {
        let fields = check_sample_trace(&output.stderr, id, credential.credential());
        println!("loaded_sample ordinal={ordinal} write_start_ns={write_start} write_end_ns={write_end} sample_start_ns={sample_start} sample_end_ns={sample_end} total_ns={} host_ns={} rpc_ns={} memory_ns={} enforcement_ns={} wait_ns={} assembly_ns={} profile_ns={} service_ns={} keys={} versions={} segments={}", timing.total_ns, timing.host_ns, timing.rpc_ns, timing.memory_ns, timing.enforcement_ns, fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6]);
    }
    println!("loaded_summary acknowledged={acknowledged} samples={} overlapping_client_intervals={overlapping} failures={failures:?}; unqualified synthetic workload", observations.len());
    assert!(
        failures.is_empty(),
        "loaded characterization failed: {failures:?}"
    );
    assert_eq!(acknowledged, RECORDS);
    assert_eq!(observations.len(), (RECORDS / PERIOD) as usize);
    assert!(
        overlapping > 0,
        "no client-observed diagnostic/write overlap"
    );
    let (mut restarted, client, _) = estate.start_for(clock, Duration::from_secs(300));
    work.client = client;
    runtime.block_on(async {
        for ordinal in [0, 1_333, 2_666, RECORDS - 1] {
            let seed = work.seed_record(ordinal).unwrap();
            work.verify(&seed).await.unwrap();
            assert!(work.execute(&seed).await.unwrap().idempotent_replay);
        }
    });
    restarted.terminate();
    restarted.finish().expect("reap reopened server");
}

#[test]
#[ignore = "explicit installed controlled conflict oracle"]
fn installed_disjoint_writes_from_one_stamp_conflict_without_partial_effect() {
    use rrd_contract::{
        AbortTransaction, BeginTransaction, CanonicalId, ErrorCode, TransactionState,
    };

    fn mutation_options(id: &str) -> RequestOptions {
        RequestOptions::new(id, id, Some(id), Some(u64::MAX)).unwrap()
    }

    let estate = Estate::install("fresh");
    let clock = Instant::now();
    let (child, client, _) = estate.start_for(clock, Duration::from_secs(300));
    let credential = rrd_engine::RrdEngine::read_installed_api_key(&estate.project).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let session = client
            .create_session(
                credential.principal_id.clone(),
                credential.credential(),
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 300_000,
                        absolute_timeout_ms: 600_000,
                        max_open_transactions: 4,
                    },
                },
                mutation_options("conflict-oracle-session"),
            )
            .await
            .unwrap();
        let mut work = Workload {
            native_worker: Default::default(),
            client,
            session,
            valid_at: estate.at,
            dimensions: 16,
            seed: 73,
            records: 8,
            read_distribution: ReadKeyDistribution::Uniform {},
            edges_per_record: 3,
            vectors_per_record: 2,
            payload_bytes: 1024,
            query_budget: serde_json::from_value(estate.plan["configuration"]["query"].clone())
                .unwrap(),
        };
        work.initialize().await.unwrap();
        let first = work.seed_record(0).unwrap();
        let second = work.seed_record(1).unwrap();
        let begin = || BeginTransaction {
            scope: CanonicalId::new("data").unwrap(),
            timeout_ms: 60_000,
        };
        let left = work
            .client
            .begin_transaction(
                &work.session,
                begin(),
                mutation_options("conflict-left-begin"),
            )
            .await
            .unwrap();
        let right = work
            .client
            .begin_transaction(
                &work.session,
                begin(),
                mutation_options("conflict-right-begin"),
            )
            .await
            .unwrap();
        assert_eq!(left.read_cursor, right.read_cursor);
        let accepted = work
            .client
            .commit_transaction(
                &work.session,
                &left.transaction_id,
                first.mutation.clone().unwrap(),
                mutation_options("conflict-left-commit"),
            )
            .await
            .unwrap();
        let published_cursor = accepted.last_runtime_cursor.unwrap();
        assert!(published_cursor > left.read_cursor);
        let rejected = work
            .client
            .commit_transaction(
                &work.session,
                &right.transaction_id,
                second.mutation.clone().unwrap(),
                mutation_options("conflict-right-stale-commit"),
            )
            .await
            .unwrap_err();
        match rejected {
            rrd_client::Error::Api { status, error } => {
                assert_eq!(error.code, ErrorCode::Conflict);
                assert!(
                    !error.retryable,
                    "a stale commit cannot be retried as the same transaction"
                );
                assert_eq!(
                    error
                        .details
                        .get(&CanonicalId::new("reason").unwrap())
                        .map(String::as_str),
                    Some("transaction-snapshot-stale")
                );
                assert_eq!(
                    error
                        .details
                        .get(&CanonicalId::new("next-action").unwrap())
                        .map(String::as_str),
                    Some("abort-and-reexecute-transaction")
                );
                assert_eq!(
                    error
                        .details
                        .get(&CanonicalId::new("expected-cursor").unwrap()),
                    Some(&right.read_cursor.to_string())
                );
                assert_eq!(
                    error
                        .details
                        .get(&CanonicalId::new("actual-cursor").unwrap()),
                    Some(&published_cursor.to_string())
                );
                let api = rrd_client::Error::Api { status, error };
                let trace = workload::runtime_conflict_trace(
                    &api,
                    "synthetic-conflict-key",
                    right.read_cursor,
                    &std::sync::atomic::AtomicUsize::new(0),
                )
                .expect("real stale response must retain bounded cursor diagnostics");
                assert!(trace.contains(&format!(
                    "read_cursor={} expected_cursor={} actual_cursor={}",
                    right.read_cursor, right.read_cursor, published_cursor
                )));
                assert!(!trace.contains("synthetic-conflict-key"));
                assert!(workload::client_error(api)
                    .starts_with("api-conflict-storage-runtime-conflict:"));
            }
            other => panic!("unexpected typed error: {other:?}"),
        }
        let repeated = work
            .client
            .commit_transaction(
                &work.session,
                &right.transaction_id,
                second.mutation.clone().unwrap(),
                mutation_options("conflict-right-stale-commit"),
            )
            .await
            .unwrap_err();
        match repeated {
            rrd_client::Error::Api { error, .. } => {
                assert_eq!(error.code, ErrorCode::Conflict);
                assert!(!error.retryable);
                assert_eq!(
                    error
                        .details
                        .get(&CanonicalId::new("reason").unwrap())
                        .map(String::as_str),
                    Some("transaction-snapshot-stale")
                );
            }
            other => panic!("unexpected repeated error: {other:?}"),
        }
        let aborted = work
            .client
            .abort_transaction(
                &work.session,
                &right.transaction_id,
                AbortTransaction {},
                mutation_options("conflict-right-abort"),
            )
            .await
            .unwrap();
        assert_eq!(aborted.state, TransactionState::Aborted);
        work.verify(&first).await.unwrap();
        work.verify_absent(&second.key).await.unwrap();
        let fresh = work.execute(&second).await.unwrap();
        assert!(!fresh.idempotent_replay);
        work.verify(&second).await.unwrap();
        super::kill(child, clock);
        let (reopened, client, _) = estate.start_for(clock, Duration::from_secs(300));
        work.client = client;
        work.verify(&first).await.unwrap();
        work.verify(&second).await.unwrap();
        let replay = work.execute(&second).await.unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.receipt_sha256, fresh.receipt_sha256);
        super::kill(reopened, clock);
    });
}

#[test]
#[ignore = "explicit native capacity-driver smoke; candidate portability job"]
fn installed_capacity_workload_smoke() {
    let estate = Estate::install("fresh");
    let clock = Instant::now();
    let (child, client, _) = estate.start_for(clock, Duration::from_secs(300));
    let credential = rrd_engine::RrdEngine::read_installed_api_key(&estate.project).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let lease = client
            .create_session(
                credential.principal_id.clone(),
                credential.credential(),
                CreateSession {
                    limits: SessionLimits {
                        idle_timeout_ms: 300_000,
                        absolute_timeout_ms: 600_000,
                        max_open_transactions: 4,
                    },
                },
                RequestOptions::mutation(
                    "capacity-smoke-session",
                    "capacity-smoke-session",
                    "capacity-smoke-session",
                )
                .unwrap(),
            )
            .await
            .unwrap();
        let mut work = Workload {
            native_worker: Default::default(),
            client,
            session: lease,
            valid_at: estate.at,
            dimensions: 16,
            seed: 73,
            records: 8,
            read_distribution: ReadKeyDistribution::Uniform {},
            edges_per_record: 3,
            vectors_per_record: 2,
            payload_bytes: 1024,
            query_budget: serde_json::from_value(estate.plan["configuration"]["query"].clone())
                .unwrap(),
        };
        work.initialize().await.unwrap();
        let mut receipts = Vec::new();
        for ordinal in 0..work.records {
            let operation = work.seed_record(ordinal).unwrap();
            let receipt = work.execute(&operation).await.unwrap();
            assert!(!receipt.idempotent_replay);
            receipts.push(receipt.receipt_sha256);
        }
        // For seed 73 and 16 all-one query dimensions, vector 1 of seed 7
        // uniquely maximizes dot product: sum(81..=96) / 256 = 5.53125.
        let winning_key = "seed-00000000000000000007";
        let before = work.verify_joined_search(winning_key, 1, 7, 5.53125).await.unwrap();
        super::kill(child, clock);
        let (mut child, client, _) = estate.start_for(clock, Duration::from_secs(300));
        work.client = client;
        let after = work.verify_joined_search(winning_key, 1, 7, 5.53125).await.unwrap();
        assert_eq!(after.known_at_cursor, before.known_at_cursor);
        assert_eq!(after.hits, before.hits);
        let replay = work.execute(&work.seed_record(7).unwrap()).await.unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.receipt_sha256, receipts[7]);
        let mut verified = 0;
        for first in (0..work.records).step_by(2) {
            let results = corpus::verify_window(
                &work,
                first,
                (first + 2).min(work.records),
                2,
                Duration::from_secs(30),
            )
            .await
            .unwrap();
            for (ordinal, effect) in results {
                assert_eq!(ordinal, verified);
                assert_eq!(effect, work.seed_record(ordinal).unwrap().effect_sha256);
                verified += 1;
            }
        }
        assert_eq!(verified, work.records);
        for (index, kind) in [
            StorageLoadKind::PointRead,
            StorageLoadKind::RangeRead,
            StorageLoadKind::Maintenance,
        ]
        .into_iter()
        .enumerate()
        {
            let operation = work
                .prepare(format!("smoke-read-{index}"), kind, index as u64)
                .unwrap();
            work.execute(&operation).await.unwrap();
            work.verify(&operation).await.unwrap();
        }
        // Exercise the campaign sampler against the owned native process as
        // well as the independent public diagnostics request below.
        let sampled = sample(&work, child.id(), clock, &None).await.unwrap();
        assert_eq!(sampled.pid, child.id());
        assert!(sampled.elapsed_ns <= ns(clock));
        assert!(sampled.storage.as_ref().is_some_and(|storage|
            storage.rrflow_kv.is_some() && storage.admission.is_some()));
        // Hold an owned asynchronous sample while a real engine request and
        // its independent readback make progress, then acquire the sample.
        let (release_sample, held_sample) = tokio::sync::oneshot::channel();
        let (sample_started, started_sample) = tokio::sync::oneshot::channel();
        let sample_work = work.clone();
        let sample_pid = child.id();
        let mut sampling = sample_task::SampleTask::new();
        sampling.start(async move {
            sample_started.send(()).map_err(|_| "smoke sample observer closed".to_owned())?;
            held_sample.await.map_err(|_| "smoke sample cancelled".to_owned())?;
            sample(&sample_work, sample_pid, clock, &None).await
        }).unwrap();
        started_sample.await.unwrap();
        let during_sample = work.prepare("smoke-sample-followup".into(), StorageLoadKind::PointRead, 2).unwrap();
        work.execute(&during_sample).await.unwrap();
        work.verify(&during_sample).await.unwrap();
        assert!(sampling.is_pending());
        release_sample.send(()).unwrap();
        assert_eq!(sampling.finish().await.unwrap().unwrap().pid, sample_pid);
        // This disposable contract-fixture campaign exercises the actual
        // scheduled segment. Its synthetic identity is not qualification.
        let campaign = smoke_campaign();
        let evidence_root = tempfile::tempdir().unwrap();
        let evidence = evidence_root.path().join("segment-evidence");
        let writer = CampaignWriter::create(&evidence, campaign.clone()).unwrap();
        writer.admit_next().unwrap();
        let mut point_run = PointRun {
            campaign: &campaign,
            point: &campaign.points[0],
            workload: work.clone(),
            pid: child.id(),
            clock,
            allocation: None,
            generations: vec![0; 6],
            previous_resources: Vec::new(),
            backup: management::ManagementTask::new(),
            sample_gate: None,
        };
        let segment = point_run.segment(&writer, LoadPhase::Warmup, ns(clock), 0, 6, true).await.unwrap();
        assert_eq!(segment.attempts.len(), 6);
        assert!(segment.resources.len() >= 3);
        assert!(segment.resources.iter().all(|value| value.pid == child.id()));
        writer.checkpoint_progress().unwrap();
        let progress = std::fs::read_to_string(evidence.join("progress.jsonl")).unwrap();
        let events: Vec<ProgressEvent> = progress.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        let resources: Vec<_> = events.iter().filter_map(|event| match event {
            ProgressEvent::Resource { segment: 0, sample } => Some(serde_json::to_value(sample).unwrap()),
            _ => None,
        }).collect();
        let expected_resources: Vec<_> = segment.resources.iter().map(|sample| serde_json::to_value(sample).unwrap()).collect();
        assert_eq!(resources, expected_resources);
        let mut outcomes: Vec<_> = events.iter().filter_map(|event| match event {
            ProgressEvent::LoadOutcome { segment: 0, attempt } => Some(attempt),
            _ => None,
        }).collect();
        outcomes.sort_by_key(|attempt| attempt.sequence);
        assert_eq!(serde_json::to_value(outcomes).unwrap(), serde_json::to_value(&segment.attempts).unwrap());
        // A later segment's initial sample must not hold scheduled offers.
        // No carried samples are supplied, making the first Resource event a
        // deterministic distinction from the old synchronous initial sample.
        let mut initial_campaign = smoke_campaign();
        initial_campaign.schedule.sample_ms = 1_000u64.try_into().unwrap();
        let mut host = sysinfo::System::new();
        host.refresh_memory();
        let host_bytes = host.total_memory();
        assert!(host_bytes > 0, "real host memory is required for this installed fixture");
        initial_campaign.storage.budgets.host_bytes = host_bytes.try_into().unwrap();
        initial_campaign.allocation.host_bytes = host_bytes.try_into().unwrap();
        initial_campaign.storage.budgets.retained_bytes = (16 * 1024 * 1024u64).try_into().unwrap();
        initial_campaign.required_lanes.push(CapacityLane::ReaderPinInterference);
        initial_campaign.reader_selection = Some(ReaderSelection::WorkloadRecords {});
        let initial_evidence = evidence_root.path().join("initial-sample-segment");
        let initial_writer = CampaignWriter::create(&initial_evidence, initial_campaign.clone()).unwrap();
        initial_writer.admit_next().unwrap();
        let initial_gate = std::sync::Arc::new(sample_task::SampleGate::default());
        let initial_clock = Instant::now() - Duration::from_secs(10);
        let mut initial_run = PointRun {
            campaign: &initial_campaign, point: &initial_campaign.points[0],
            workload: work.clone(), pid: child.id(), clock: initial_clock, allocation: None,
            generations: vec![0; 5], previous_resources: Vec::new(),
            backup: management::ManagementTask::new(), sample_gate: Some(initial_gate.clone()),
        };
        // Sequences 15-18 are real writes; avoid sequence 14's backup lane.
        let origin = ns(initial_clock).checked_sub(3_000_000_000).unwrap();
        let mut initial_segment = Box::pin(initial_run.segment(&initial_writer, LoadPhase::Measured, origin, 15, 5, true));
        tokio::time::timeout(Duration::from_secs(20), async {
            tokio::select! {
                _ = initial_gate.entered.notified() => {}
                result = &mut initial_segment => panic!("segment ended before initial sample gate: {result:?}"),
            }
        }).await.unwrap();
        initial_writer.checkpoint_progress().unwrap();
        let progress = std::fs::read_to_string(initial_evidence.join("progress.jsonl")).unwrap();
        assert!(!progress.lines().map(|line| serde_json::from_str::<ProgressEvent>(line).unwrap())
            .any(|event| matches!(event, ProgressEvent::Resource { .. })), "initial sample completed before its gate");
        let acknowledged = tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                tokio::select! {
                    result = &mut initial_segment => panic!("segment ended while initial sample held: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(50)) => {
                        initial_writer.checkpoint_progress().unwrap();
                        let raw = std::fs::read_to_string(initial_evidence.join("progress.jsonl")).unwrap();
                        let events: Vec<ProgressEvent> = raw.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
                        if let Some(attempt) = events.iter().find_map(|event| match event {
                            ProgressEvent::LoadOutcome { attempt, .. }
                                if attempt.outcome == LoadOutcome::Acknowledged
                                    && events.iter().any(|other| matches!(other,
                                        ProgressEvent::Readback { observation: Readback::Present { logical_id, .. }, .. }
                                            if logical_id == &attempt.logical_id)) => Some(attempt.clone()),
                            _ => None,
                        }) { break attempt; }
                    }
                }
            }
        }).await.unwrap();
        assert!(!initial_gate.dropped.load(std::sync::atomic::Ordering::SeqCst));
        initial_gate.open();
        let initial_segment = tokio::time::timeout(Duration::from_secs(30), initial_segment).await.unwrap().unwrap();
        assert!(initial_segment.attempts.iter().any(|attempt| attempt.sequence == acknowledged.sequence));
        assert!(!initial_segment.resources.is_empty());
        let reader = initial_segment.reader.as_ref().expect("reader remains mandatory");
        reader.validate().unwrap();
        assert!(reader.opened_ns <= acknowledged.started_ns && acknowledged.finished_ns <= reader.observed_ns);
        validate_segment(&initial_campaign, &initial_segment).unwrap_or_else(|error| {
            let retained: Vec<_> = initial_segment.resources.iter().map(|sample| &sample.retained_bytes).collect();
            panic!("segment validation failed: {error}; retained samples: {retained:?}")
        });
        let mut missing_samples = initial_segment.clone();
        missing_samples.resources.clear();
        assert_eq!(validate_segment(&initial_campaign, &missing_samples).unwrap_err(),
            "segment evidence missing or exceeds bound");
        // Hold a harness-only asynchronous sample in the actual scheduled
        // segment. A later real ACK must be journaled before the gate opens;
        // the completed segment must contain that exact acknowledged attempt.
        let mut held_campaign = smoke_campaign();
        held_campaign.schedule.warmup_ms = 5_000u64.try_into().unwrap();
        held_campaign.evidence.segment_attempts = 20u64.try_into().unwrap();
        let held_evidence = evidence_root.path().join("held-segment-evidence");
        let held_writer = CampaignWriter::create(&held_evidence, held_campaign.clone()).unwrap();
        held_writer.admit_next().unwrap();
        let held_gate = std::sync::Arc::new(sample_task::SampleGate::default());
        let mut held_run = PointRun {
            campaign: &held_campaign,
            point: &held_campaign.points[0],
            workload: work.clone(),
            pid: child.id(), clock, allocation: None, generations: vec![0; 20],
            previous_resources: Vec::new(), backup: management::ManagementTask::new(),
            sample_gate: Some(held_gate.clone()),
        };
        let mut held_segment = Box::pin(held_run.segment(&held_writer, LoadPhase::Warmup, ns(clock), 0, 20, true));
        tokio::time::timeout(Duration::from_secs(20), async {
            tokio::select! {
                _ = held_gate.entered.notified() => {}
                result = &mut held_segment => panic!("segment ended before held sample: {result:?}"),
            }
        }).await.unwrap();
        let held_ack = tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                tokio::select! {
                    result = &mut held_segment => panic!("segment ended while sample was held: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        held_writer.checkpoint_progress().unwrap();
                        let raw = std::fs::read_to_string(held_evidence.join("progress.jsonl")).unwrap();
                        let acknowledged = raw.lines().map(|line| serde_json::from_str::<ProgressEvent>(line).unwrap()).find_map(|event|
                            match event {
                                ProgressEvent::LoadOutcome { attempt, .. }
                                    if attempt.sequence >= 2 && attempt.outcome == LoadOutcome::Acknowledged => Some(attempt),
                                _ => None,
                            });
                        if let Some(attempt) = acknowledged { break attempt; }
                    }
                }
            }
        }).await.unwrap();
        assert!(!held_gate.dropped.load(std::sync::atomic::Ordering::SeqCst));
        // Move the hold into the real native helper after the async gate.
        // Record a new ACK after observing that helper's entry, rather than
        // counting an earlier event as proof of concurrent scheduling.
        let native_marker = evidence_root.path().join("held-native-worker");
        work.native_worker.hold_after(0, native_marker.clone());
        held_gate.open();
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    result = &mut held_segment => panic!("segment ended before native hold: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(5)) => {
                        if native_marker.exists() { break; }
                    }
                }
            }
        }).await.unwrap();
        held_writer.checkpoint_progress().unwrap();
        let before_native = std::fs::read_to_string(held_evidence.join("progress.jsonl")).unwrap();
        let max_before = before_native.lines().map(|line| serde_json::from_str::<ProgressEvent>(line).unwrap())
            .filter_map(|event| match event {
                ProgressEvent::LoadOutcome { attempt, .. }
                    if attempt.outcome == LoadOutcome::Acknowledged => Some(attempt.sequence),
                _ => None,
            }).max().unwrap_or(0);
        let native_ack = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    result = &mut held_segment => panic!("segment ended while native helper was held: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(25)) => {
                        held_writer.checkpoint_progress().unwrap();
                        let raw = std::fs::read_to_string(held_evidence.join("progress.jsonl")).unwrap();
                        if let Some(attempt) = raw.lines().map(|line| serde_json::from_str::<ProgressEvent>(line).unwrap())
                            .find_map(|event| match event {
                                ProgressEvent::LoadOutcome { attempt, .. }
                                    if attempt.sequence > max_before && attempt.outcome == LoadOutcome::Acknowledged => Some(attempt),
                                _ => None,
                            }) { break attempt; }
                    }
                }
            }
        }).await.unwrap();
        std::fs::write(native_marker.with_extension("release"), b"release").unwrap();
        let held_result = tokio::time::timeout(Duration::from_secs(20), &mut held_segment).await.unwrap().unwrap();
        drop(held_segment);
        assert_eq!(held_result.attempts.len(), 20);
        assert!(held_result.attempts.iter().any(|attempt|
            attempt.sequence == native_ack.sequence && attempt.request_sha256 == native_ack.request_sha256
                && attempt.result_sha256 == native_ack.result_sha256 && attempt.outcome == LoadOutcome::Acknowledged));
        assert!(held_result.resources.len() >= 3);
        assert!(held_result.resources.iter().all(|sample| sample.pid == child.id()));
        assert!(held_result.attempts.iter().any(|attempt|
            attempt.sequence == held_ack.sequence && attempt.request_sha256 == held_ack.request_sha256
                && attempt.result_sha256 == held_ack.result_sha256 && attempt.outcome == LoadOutcome::Acknowledged));
        held_writer.checkpoint_progress().unwrap();
        let raw = std::fs::read_to_string(held_evidence.join("progress.jsonl")).unwrap();
        let events: Vec<ProgressEvent> = raw.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        let resources: Vec<_> = events.iter().filter_map(|event| match event {
            ProgressEvent::Resource { segment: 0, sample } => Some(serde_json::to_value(sample).unwrap()),
            _ => None,
        }).collect();
        let expected_resources: Vec<_> = held_result.resources.iter().map(|sample| serde_json::to_value(sample).unwrap()).collect();
        assert_eq!(resources, expected_resources);
        let mut outcomes: Vec<_> = events.iter().filter_map(|event| match event {
            ProgressEvent::LoadOutcome { segment: 0, attempt } => Some(attempt),
            _ => None,
        }).collect();
        outcomes.sort_by_key(|attempt| attempt.sequence);
        assert_eq!(serde_json::to_value(outcomes).unwrap(), serde_json::to_value(&held_result.attempts).unwrap());
        println!("held_segment_summary async_ack_sequence={} native_ack_sequence={} attempts={} resources={} checkpoint_matches=true", held_ack.sequence, native_ack.sequence, held_result.attempts.len(), held_result.resources.len());
        // Interrupt a real segment while its harness-only async sample is held.
        // Only independently verified ACKs are replayed after an owned restart.
        let mut interrupted_campaign = smoke_campaign();
        interrupted_campaign.schedule.warmup_ms = 5_000u64.try_into().unwrap();
        interrupted_campaign.evidence.segment_attempts = 20u64.try_into().unwrap();
        let interrupted_evidence = evidence_root.path().join("interrupted-evidence");
        let interrupted_writer = CampaignWriter::create(&interrupted_evidence, interrupted_campaign.clone()).unwrap();
        interrupted_writer.admit_next().unwrap();
        let gate = std::sync::Arc::new(sample_task::SampleGate::default());
        let mut interrupted_run = PointRun {
            campaign: &interrupted_campaign,
            point: &interrupted_campaign.points[0],
            workload: work.clone(),
            pid: child.id(), clock, allocation: None, generations: vec![0; 20],
            previous_resources: Vec::new(), backup: management::ManagementTask::new(),
            sample_gate: Some(gate.clone()),
        };
        let mut interrupted = Box::pin(interrupted_run.segment(&interrupted_writer, LoadPhase::Warmup, ns(clock), 0, 20, true));
        tokio::time::timeout(Duration::from_secs(20), async {
            tokio::select! {
                _ = gate.entered.notified() => {}
                result = &mut interrupted => panic!("segment ended before sample gate: {result:?}"),
            }
        }).await.unwrap();
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                tokio::select! {
                    result = &mut interrupted => panic!("segment ended while sample was held: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(200)) => {
                        interrupted_writer.checkpoint_progress().unwrap();
                        let raw = std::fs::read_to_string(interrupted_evidence.join("progress.jsonl")).unwrap();
                        let seen = raw.lines().map(|line| serde_json::from_str::<ProgressEvent>(line).unwrap()).any(|event|
                            matches!(event, ProgressEvent::LoadOutcome { attempt, .. }
                                if attempt.write.is_some() && attempt.outcome == LoadOutcome::Acknowledged));
                        if seen { break; }
                    }
                }
            }
        }).await.unwrap();
        // Keep the original async interruption oracle, then move the held
        // acquisition into the native helper and interrupt that real segment.
        let cancel_marker = evidence_root.path().join("cancelled-native-worker");
        work.native_worker.hold_after(0, cancel_marker.clone());
        gate.open();
        let helper_pid = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    result = &mut interrupted => panic!("segment ended before native interruption: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(5)) => {
                        if let Ok(raw) = std::fs::read_to_string(&cancel_marker) {
                            if let Ok(pid) = raw.parse::<u32>() { break pid; }
                        }
                    }
                }
            }
        }).await.unwrap();
        drop(interrupted);
        work.native_worker.shutdown().await.unwrap();
        let mut processes = sysinfo::System::new();
        let helper = sysinfo::Pid::from_u32(helper_pid);
        processes.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[helper]), true);
        assert!(processes.process(helper).is_none(), "interrupted native helper survived");
        tokio::time::timeout(Duration::from_secs(2), async {
            while !gate.dropped.load(std::sync::atomic::Ordering::SeqCst) { tokio::task::yield_now().await; }
        }).await.unwrap();
        interrupted_writer.checkpoint_progress().unwrap();
        let raw = std::fs::read_to_string(interrupted_evidence.join("progress.jsonl")).unwrap();
        let interrupted_events: Vec<ProgressEvent> = raw.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        let acknowledged: Vec<_> = interrupted_events.iter().filter_map(|event| match event {
            ProgressEvent::LoadOutcome { attempt, .. } if attempt.write.is_some() && attempt.outcome == LoadOutcome::Acknowledged => Some(attempt.clone()),
            _ => None,
        }).collect();
        assert!(!acknowledged.is_empty());
        assert_eq!(interrupted_events.iter().filter(|event| matches!(event, ProgressEvent::Resource { .. })).count(), 1);
        super::kill(child, clock);
        let (restarted, client, _) = estate.start_for(clock, Duration::from_secs(300));
        child = restarted;
        work.client = client;
        assert_eq!(std::fs::read_to_string(interrupted_evidence.join("progress.jsonl")).unwrap(), raw);
        for attempt in &acknowledged {
            let operation = work.prepare_load(format!("load-0-{}", attempt.sequence), attempt.write.unwrap()).unwrap();
            assert_eq!(operation.request_sha256, attempt.request_sha256);
            work.verify(&operation).await.unwrap();
            let replay = work.execute(&operation).await.unwrap();
            assert!(replay.idempotent_replay);
            assert_eq!(replay.receipt_sha256, attempt.result_sha256);
        }
        let physical = work.client.read_storage_diagnostics(&work.session,
            ReadStorageDiagnostics {scope: format!("instance:{}",work.client.instance_id())},
            RequestOptions::read("smoke-storage", "smoke-storage").unwrap()).await.unwrap();
        assert!(physical.rrflow_kv.is_some() && physical.admission.is_some());
        let mut final_states = Vec::new();
        for slot in 0..3 {
            for generation in 0..=slot+2 {
                let operation = work.prepare_load(format!("smoke-slot-{slot}-{generation}"),
                    WriteTransition { slot, generation }).unwrap();
                let receipt = work.execute(&operation).await.unwrap();
                assert!(!receipt.idempotent_replay);
                work.verify(&operation).await.unwrap();
                if generation == slot+2 { final_states.push((operation, receipt.receipt_sha256)); }
            }
        }
        let write = work
            .prepare(
                "smoke-concurrent-write".into(),
                StorageLoadKind::MixedWrite,
                99,
            )
            .unwrap();
        let retained_reader = reader::HeldReader::open(&work, 0, clock, None).await.unwrap();
        let mut management = management::ManagementTask::new();
        let (release_receipt, held_receipt) = tokio::sync::oneshot::channel();
        let backup_work = work.clone();
        management.start(Instant::now() + Duration::from_secs(30), async move {
            let receipt = lifecycle::backup(backup_work, "smoke-backup".into(), clock).await?;
            // Hold only the verified receipt handoff, not engine execution. This
            // makes the owned pending-task boundary deterministic in native CI.
            held_receipt.await.map_err(|_| "smoke receipt handoff cancelled".to_owned())?;
            Ok(receipt)
        }).unwrap();
        let written = work.execute(&write).await.unwrap();
        assert!(management.try_finish().unwrap().is_none());
        let followup = work.prepare("smoke-management-followup".into(), StorageLoadKind::PointRead, 1).unwrap();
        work.execute(&followup).await.unwrap();
        work.verify(&followup).await.unwrap();
        release_receipt.send(()).unwrap();
        let backup = management.finish().await.unwrap().unwrap();
        assert!(!management.is_pending());
        let reader_evidence = retained_reader.finish(&work, clock).await.unwrap();
        work.verify(&write).await.unwrap();
        let selected = reader::HeldReader::open(
            &work, 1, clock, Some(&ReaderSelection::WorkloadRecords {}),
        ).await.unwrap();
        let selected_write = work.prepare("smoke-selected-write".into(), StorageLoadKind::MixedWrite, 101).unwrap();
        let selected_receipt = work.execute(&selected_write).await.unwrap();
        work.verify(&selected_write).await.unwrap();
        let selected_evidence = selected.finish(&work, clock).await.unwrap();
        assert!(selected_evidence.opened.rows > 0);
        println!(
            "selected_reader_interference={}",
            serde_json::to_string(&selected_evidence).unwrap()
        );
        let killed = super::kill(child, clock);
        let (reopened, client, identity) = estate.start_for(clock, Duration::from_secs(300));
        work.client = client;
        // Exercise the production post-reopen bounded readback before replay.
        let mut reopened_digest =
            digest("c07g.reopened-corpus", &(work.seed, work.records)).unwrap();
        let mut expected_digest = reopened_digest.clone();
        for first in (0..work.records).step_by(2) {
            let (next_digest, values) = corpus::verify_reopened_window(
                &work,
                first,
                (first + 2).min(work.records),
                2,
                Duration::from_secs(30),
                reopened_digest,
            )
            .await
            .unwrap();
            for (ordinal, effect) in values {
                assert_eq!(effect, work.seed_record(ordinal).unwrap().effect_sha256);
                expected_digest =
                    digest("c07g.reopened-corpus", &(expected_digest, ordinal, &effect)).unwrap();
            }
            reopened_digest = next_digest;
        }
        assert_eq!(reopened_digest, expected_digest);
        // Every old value is read before replay; exact old request identities
        // must return the stored receipt after this real process boundary.
        for ordinal in 0..work.records {
            let operation = work.seed_record(ordinal).unwrap();
            work.verify(&operation).await.unwrap();
            let replay = work.execute(&operation).await.unwrap();
            assert!(replay.idempotent_replay);
            assert_eq!(replay.receipt_sha256, receipts[ordinal as usize]);
        }
        for (operation, original) in &final_states {
            work.verify(operation).await.unwrap();
            let replay = work.execute(operation).await.unwrap();
            assert!(replay.idempotent_replay);
            assert_eq!(&replay.receipt_sha256, original);
        }
        work.verify(&write).await.unwrap();
        let replay = work.execute(&write).await.unwrap();
        assert!(replay.idempotent_replay);
        assert_eq!(replay.receipt_sha256, written.receipt_sha256);
        work.verify(&selected_write).await.unwrap();
        let selected_replay = work.execute(&selected_write).await.unwrap();
        assert!(selected_replay.idempotent_replay);
        assert_eq!(selected_replay.receipt_sha256, selected_receipt.receipt_sha256);
        println!(
            "installed_capacity_workload_smoke={}",
            serde_json::json!({
            "build":estate.build,"records":8,"edges":24,"vectors":16,"dimensions":16,
            "replayed_transactions":12,"write_transitions":{"insert":3,"update":4,"retire":3,"recreate":2},"backup_sha256":backup.receipt.backup.backup_sha256,
            "management_followup_verified":true,"reader_interference":reader_evidence,"termination":killed,"reopened":identity,
            "interrupted_acknowledged_writes":acknowledged.len(),"interrupted_sample_cancelled":gate.dropped.load(std::sync::atomic::Ordering::SeqCst),
            "capacity_qualification":false})
        );
        super::kill(reopened, clock);
    });
}

async fn run(campaign: &StorageCampaign, writer: &mut CampaignWriter) -> DriverResult<()> {
    run_estate(campaign, writer, None).await
}
async fn run_estate(
    campaign: &StorageCampaign,
    writer: &mut CampaignWriter,
    fleet: Option<&fleet::Participation>,
) -> DriverResult<()> {
    if campaign
        .required_lanes
        .iter()
        .any(|lane| *lane == CapacityLane::Cold && !cfg!(target_os = "linux"))
    {
        return Err("native cold-cache control is not available on this platform".into());
    }
    if campaign
        .required_lanes
        .contains(&CapacityLane::BackupUnderWrite)
        && campaign.points.iter().any(|point| {
            u128::from(campaign.schedule.measured_ms.get())
                * u128::from(point.requests_per_second.get())
                < 15_000
        })
    {
        return Err(
            "backup interference requires the measured mix to reach its first mixed write".into(),
        );
    }
    if campaign.schedule.operation_deadline_ms.get() > 60_000 {
        return Err("installed request deadline exceeds the driver's one-minute bound".into());
    }
    if campaign.allocation.estates.get() != 1 {
        return Err("fleet requires native aggregate estate driver".into());
    }
    if campaign
        .required_lanes
        .contains(&CapacityLane::ReaderPinInterference)
        && campaign.schedule.operation_deadline_ms.get() > 30_000
    {
        return Err("retained-reader lane requires operation deadline at most 30 seconds".into());
    }
    let allocation = native::receipt(campaign)?;
    if campaign.purpose == CampaignPurpose::Qualification && allocation.is_none() {
        return Err("qualification requires launcher-owned native allocation receipt".into());
    }
    let configuration = std::env::var_os("RRFLOW_SUSTAINED_CONFIGURATION").map(PathBuf::from);
    let clock = fleet.map_or_else(Instant::now, |fleet| fleet.clock);
    for point in &campaign.points {
        for seed in &campaign.seeds {
            writer.admit_next()?;
            let estate = Estate::install_configured("fresh", configuration.as_deref());
            if estate.build["executable_sha256"].as_str()
                != Some(campaign.storage.candidate.executable_sha256.as_str())
                || estate.plan["configuration"]["configuration_sha256"].as_str()
                    != Some(campaign.storage.candidate.configuration_sha256.as_str())
            {
                return Err("installed binary/configuration differs from frozen campaign".into());
            }
            let lifetime_ms = [
                LoadPhase::Warmup,
                LoadPhase::Measured,
                LoadPhase::Burst,
                LoadPhase::Drain,
                LoadPhase::Soak,
            ]
            .into_iter()
            .try_fold(
                campaign
                    .schedule
                    .setup_ms
                    .get()
                    .checked_add(campaign.schedule.lifecycle_ms.get())
                    .and_then(|v| v.checked_add(600_000))
                    .ok_or("setup deadline overflow")?,
                |n, p| n.checked_add(campaign.schedule.duration_ms(p)),
            )
            .ok_or("campaign deadline overflow")?
            .checked_add(fleet.map_or(0, |f| f.quiet_extra_ms(campaign.schedule.measured_ms.get())))
            .ok_or("fleet isolated baseline deadline overflow")?;
            let (child, client, _) = estate.start_for(clock, Duration::from_millis(lifetime_ms));
            native::enforcement(child.id(), allocation.as_ref())?;
            let mut child = Some(child);
            let mut session_ordinal = 0;
            let mut session_opened = Instant::now();
            let mut work = session(
                &estate,
                client,
                *seed,
                campaign,
                point.clients.get(),
                session_ordinal,
            )
            .await?;
            let seeded = corpus::seed(
                &estate,
                &mut work,
                campaign,
                &ProgressContext { writer, point },
                clock,
                &mut session_ordinal,
                &mut session_opened,
            )
            .await;
            if seeded.is_err() {
                // Preserve a single supplemental observation before stopping
                // the owned server. This is not a workload
                // segment and cannot qualify the campaign or hide its error.
                let status = if let Some(child) = &child {
                    match tokio::time::timeout(
                        Duration::from_secs(5),
                        sample(&work, child.id(), clock, &allocation),
                    )
                    .await
                    {
                        Ok(Ok(observation)) => {
                            if writer
                                .record_event(&ProgressEvent::Resource {
                                    segment: writer.next_index(),
                                    sample: Box::new(observation),
                                })
                                .and_then(|_| writer.checkpoint_progress())
                                .is_ok()
                            {
                                "observed"
                            } else {
                                "artifact-denied"
                            }
                        }
                        Ok(Err(_)) => "unavailable",
                        Err(_) => "timeout",
                    }
                } else {
                    "unavailable"
                };
                eprintln!("native_setup_failure_observation={status}");
                if let Some(mut stopped) = child.take() {
                    stopped.terminate();
                    // finish joins the monitor and reaps the child even when
                    // bounded output capture reports a failure. Drop remains
                    // the process guard for an early monitor error.
                    let _ = stopped.finish();
                    drop(stopped);
                }
                eprintln!(
                    "native_setup_failure_estate={}",
                    serde_json::json!({"path": estate.temp.retain(), "private_state": true})
                );
            }
            let mut corpus = Some(seeded?);
            // Fresh session for the bounded lifecycle lane, avoiding inherited
            // setup age. Subsequent load still rotates joined session segments.
            session_opened = Instant::now() - Duration::from_secs(120);
            rotate_session(
                &estate,
                &mut work,
                campaign,
                point.clients.get(),
                &mut session_ordinal,
                &mut session_opened,
            )
            .await?;
            let reopens_result = lifecycle::reopens(
                &estate,
                &mut child,
                &mut work,
                campaign,
                (clock, Duration::from_millis(lifetime_ms)),
                &ProgressContext { writer, point },
                (&mut session_ordinal, &mut session_opened),
            )
            .await;
            let mut reopens = Some(retain_failed_lifecycle(&estate.temp, reopens_result)?);
            let mut cold_start = if campaign.required_lanes.contains(&CapacityLane::Cold) {
                Some(
                    cold::run(
                        &estate,
                        &mut child,
                        &mut work,
                        campaign,
                        clock,
                        Duration::from_millis(lifetime_ms),
                        &ProgressContext { writer, point },
                    )
                    .await?,
                )
            } else {
                None
            };
            let mut driver = PointRun {
                campaign,
                point,
                workload: work,
                pid: child
                    .as_ref()
                    .ok_or("owned server missing after lifecycle lane")?
                    .id(),
                clock,
                allocation: allocation.clone(),
                generations: vec![0; campaign.evidence.segment_attempts.get() as usize],
                previous_resources: Vec::new(),
                backup: management::ManagementTask::new(),
                sample_gate: None,
            };
            for phase in [
                LoadPhase::Warmup,
                LoadPhase::Measured,
                LoadPhase::Burst,
                LoadPhase::Drain,
                LoadPhase::Soak,
            ] {
                if phase == LoadPhase::Soak {
                    writer.admit_soak(&point.id, *seed)?;
                }
                let mut phase_origin = if let Some(fleet) = fleet {
                    rotate_session(
                        &estate,
                        &mut driver.workload,
                        campaign,
                        point.clients.get(),
                        &mut session_ordinal,
                        &mut session_opened,
                    )
                    .await?;
                    let stage = match phase {
                        LoadPhase::Warmup => "warmup",
                        LoadPhase::Measured => "measured",
                        LoadPhase::Burst => "burst",
                        LoadPhase::Drain => "drain",
                        LoadPhase::Soak => "soak",
                    };
                    Some(
                        fleet
                            .boundary(
                                stage,
                                &estate,
                                &mut driver,
                                &mut session_ordinal,
                                &mut session_opened,
                            )
                            .await?,
                    )
                } else {
                    None
                };
                if phase == LoadPhase::Measured {
                    if let Some(fleet) = fleet.filter(|f| f.trial == FleetTrial::Quiet) {
                        fleet
                            .wait_quiet(
                                fleet.estate,
                                &mut driver,
                                writer,
                                (&estate, &mut session_ordinal, &mut session_opened),
                            )
                            .await?;
                        phase_origin = Some(ns(clock));
                    }
                }
                if phase == LoadPhase::Drain
                    || (phase != LoadPhase::Warmup
                        && fleet.is_some_and(|f| {
                            !f.trial.active(f.estate)
                                && !(phase == LoadPhase::Measured && f.trial == FleetTrial::Quiet)
                        }))
                {
                    let mut remaining = campaign.schedule.duration_ms(phase);
                    while remaining > 0 {
                        writer.admit_next()?;
                        rotate_session(
                            &estate,
                            &mut driver.workload,
                            campaign,
                            point.clients.get(),
                            &mut session_ordinal,
                            &mut session_opened,
                        )
                        .await?;
                        let duration = campaign.idle_segment_ms(remaining)?;
                        writer.append(&driver.drain(writer, duration).await?)?;
                        remaining -= duration;
                    }
                    writer.verify_drain(&point.id, *seed)?;
                    continue;
                }
                let rate =
                    point.requests_per_second.get() * if phase == LoadPhase::Burst { 2 } else { 1 };
                let count =
                    (campaign.schedule.duration_ms(phase) as u128 * rate as u128).div_ceil(1000);
                let count: u64 = count.try_into().map_err(|_| "offered count overflow")?;
                let origin = phase_origin.unwrap_or_else(|| ns(clock));
                let mut next = 0;
                while next < count {
                    writer.admit_next()?;
                    if !driver.backup.is_pending() {
                        rotate_session(
                            &estate,
                            &mut driver.workload,
                            campaign,
                            point.clients.get(),
                            &mut session_ordinal,
                            &mut session_opened,
                        )
                        .await?;
                    }
                    let active_ms = campaign
                        .storage
                        .budgets
                        .run_ms
                        .get()
                        .checked_sub(campaign.schedule.operation_deadline_ms.get())
                        .filter(|v| *v > 0)
                        .ok_or("segment must reserve an operation deadline")?
                        .min(
                            if campaign
                                .required_lanes
                                .contains(&CapacityLane::ReaderPinInterference)
                            {
                                10_000
                            } else {
                                120_000
                            },
                        );
                    let timed_cap = (u128::from(rate) * u128::from(active_ms) / 1000)
                        .max(1)
                        .min(u128::from(u64::MAX)) as u64;
                    let take = (count - next)
                        .min(campaign.evidence.segment_attempts.get())
                        .min(timed_cap)
                        .min(
                            campaign
                                .storage
                                .budgets
                                .retained_bytes
                                .get()
                                .min(campaign.allocation.engine_bytes.get() / 4)
                                .saturating_sub(
                                    (driver.generations.len() as u64).saturating_mul(8),
                                )
                                / driver.workload.retained_request_bound().saturating_mul(2),
                        );
                    if take == 0 {
                        return Err("prepared corpus exceeds retained driver admission".into());
                    }
                    let result = tokio::time::timeout(
                        Duration::from_millis(campaign.storage.budgets.run_ms.get().min(180_000)),
                        driver.segment(writer, phase, origin, next, take, next + take == count),
                    )
                    .await
                    .map_err(|_| "installed segment exceeded its frozen deadline".to_owned())
                    .and_then(std::convert::identity);
                    let cleanup = driver.workload.native_worker.shutdown().await;
                    let mut segment = match (result, cleanup) {
                        (Ok(value), Ok(())) => value,
                        (Err(error), Ok(())) | (Ok(_), Err(error)) => return Err(error),
                        (Err(error), Err(cleanup)) => {
                            return Err(format!("{error}; native worker cleanup: {cleanup}"))
                        }
                    };
                    if phase == LoadPhase::Warmup && next == 0 {
                        segment.corpus = corpus.take();
                        segment.cold_start = cold_start.take();
                        if let Some(raw) = &segment.cold_start {
                            segment.lanes.push(LaneEvidence {
                                lane: CapacityLane::Cold,
                                repetitions: 1u64.try_into()?,
                                receipt_sha256: digest("c07g.cold-start", raw)?,
                                acknowledged_readback_verified: true,
                            });
                        }
                        segment.reopens = reopens.take();
                        if let Some(raw) = &segment.reopens {
                            segment.lanes.push(LaneEvidence {
                                lane: CapacityLane::ForcedReopens,
                                repetitions: (raw.cycles.len() as u64).try_into()?,
                                receipt_sha256: digest("c07g.reopen-evidence", raw)?,
                                acknowledged_readback_verified: true,
                            });
                        }
                    }
                    writer.append(&segment)?;
                    next += take;
                }
                if phase == LoadPhase::Measured {
                    if let Some(fleet) = fleet.filter(|f| f.trial == FleetTrial::Quiet) {
                        let all = fleet.finish_quiet()?;
                        fleet
                            .wait_quiet(
                                all,
                                &mut driver,
                                writer,
                                (&estate, &mut session_ordinal, &mut session_opened),
                            )
                            .await?;
                    }
                }
            }
            if let Some(fleet) = fleet {
                rotate_session(
                    &estate,
                    &mut driver.workload,
                    campaign,
                    point.clients.get(),
                    &mut session_ordinal,
                    &mut session_opened,
                )
                .await?;
                fleet
                    .boundary(
                        "loaded",
                        &estate,
                        &mut driver,
                        &mut session_ordinal,
                        &mut session_opened,
                    )
                    .await?;
            }
            // Reopen after offered load and verify the final generated state
            // before any retry can rewrite it. Progress retains incomplete runs.
            tokio::time::timeout(
                Duration::from_millis(campaign.schedule.lifecycle_ms.get()),
                async {
                    let started_ns = ns(clock);
                    let killed = super::kill(child.take().ok_or("missing final server")?, clock);
                    let (reopened, client, _) =
                        estate.start_for(clock, Duration::from_millis(lifetime_ms));
                    let reopened_pid = reopened.id();
                    native::enforcement(reopened_pid, allocation.as_ref())?;
                    driver.pid = reopened_pid;
                    child = Some(reopened);
                    driver.workload.client = client;
                    session_opened = Instant::now() - Duration::from_secs(120);
                    rotate_session(
                        &estate,
                        &mut driver.workload,
                        campaign,
                        point.clients.get(),
                        &mut session_ordinal,
                        &mut session_opened,
                    )
                    .await?;
                    let mut states = digest("c07g.final-state", &(*seed, point.id.clone()))?;
                    for record in 0..driver.workload.records {
                        rotate_session(
                            &estate,
                            &mut driver.workload,
                            campaign,
                            point.clients.get(),
                            &mut session_ordinal,
                            &mut session_opened,
                        )
                        .await?;
                        let operation = driver.workload.seed_record(record)?;
                        driver.workload.verify(&operation).await?;
                        states = digest("c07g.final-state", &(states, operation.effect_sha256))?;
                    }
                    let mut working_slots = 0;
                    for (slot, generation) in driver
                        .generations
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| **n > 0)
                    {
                        rotate_session(
                            &estate,
                            &mut driver.workload,
                            campaign,
                            point.clients.get(),
                            &mut session_ordinal,
                            &mut session_opened,
                        )
                        .await?;
                        let operation = driver.workload.prepare_load(
                            "final-readback".into(),
                            WriteTransition {
                                slot: slot as u64,
                                generation: generation - 1,
                            },
                        )?;
                        driver.workload.verify(&operation).await?;
                        states = digest("c07g.final-state", &(states, operation.effect_sha256))?;
                        working_slots += 1;
                    }
                    writer.record_event(&ProgressEvent::FinalReadback {
                        point: point.id.clone(),
                        seed: *seed,
                        killed_pid: killed["pid"]
                            .as_u64()
                            .ok_or("final killed PID missing")?
                            .try_into()
                            .map_err(|_| "final PID overflow")?,
                        reopened_pid,
                        seed_records: driver.workload.records,
                        working_slots,
                        states_sha256: states,
                        started_ns,
                        finished_ns: ns(clock),
                    })?;
                    writer.checkpoint_progress()?;
                    Ok::<(), String>(())
                },
            )
            .await
            .map_err(|_| "final readback exceeded lifecycle deadline")??;
            if let Some(fleet) = fleet {
                rotate_session(
                    &estate,
                    &mut driver.workload,
                    campaign,
                    point.clients.get(),
                    &mut session_ordinal,
                    &mut session_opened,
                )
                .await?;
                fleet
                    .boundary(
                        "reopened",
                        &estate,
                        &mut driver,
                        &mut session_ordinal,
                        &mut session_opened,
                    )
                    .await?;
            }
            drop(driver);
            drop(child);
        }
    }
    Ok(())
}
