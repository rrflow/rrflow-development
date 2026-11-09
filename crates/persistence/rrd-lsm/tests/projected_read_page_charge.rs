use rrd_lsm::{
    Database, DatabaseOptions, Durability, Error, Mutation, ProjectedReadBudget,
    ProjectedReadEvidence, ProjectedReadProjection, ProjectedReadRange, ProjectedReadRequest,
    ProjectedReadResource, SegmentCompressionPolicy, SegmentIoMode, SegmentIoPolicy, WriteBatch,
};

const ROWS: usize = 2_000;
const VALUE_BYTES: usize = 1_024;
const PHYSICAL: [&str; 3] = ["mapped-raw", "bounded-raw", "bounded-lz4"];

type Rows = Vec<(Vec<u8>, Vec<u8>)>;

fn options(physical: &str, page_cache_bytes: usize) -> DatabaseOptions {
    DatabaseOptions {
        page_cache_bytes,
        segment_io: SegmentIoPolicy {
            mode: if physical == "mapped-raw" {
                SegmentIoMode::Mmap
            } else {
                SegmentIoMode::Bounded
            },
            allow_fallback: false,
            ..SegmentIoPolicy::default()
        },
        segment_compression: if physical == "bounded-lz4" {
            SegmentCompressionPolicy::default()
        } else {
            SegmentCompressionPolicy::None
        },
        ..DatabaseOptions::default()
    }
}

fn row(index: usize) -> (Vec<u8>, Vec<u8>) {
    let key = format!("row/{index:06}").into_bytes();
    // Varies by row and stays compressible, so LZ4 pages really are compressed.
    let value = (0..VALUE_BYTES)
        .map(|offset| ((index + offset / 16) % 251) as u8)
        .collect();
    (key, value)
}

fn write_run(database: &mut Database, rows: impl Iterator<Item = usize>) {
    for index in rows {
        let (key, value) = row(index);
        database
            .write_owned(
                WriteBatch::new(vec![Mutation::Put { key, value }]).unwrap(),
                Durability::Authoritative,
            )
            .unwrap();
    }
    database.flush_memtable(1).unwrap().unwrap();
}

fn budget(max_batch_rows: usize) -> ProjectedReadBudget {
    ProjectedReadBudget {
        max_batch_rows,
        ..ProjectedReadBudget::default()
    }
}

fn scan(
    database: &Database,
    projection: ProjectedReadProjection,
    budget: ProjectedReadBudget,
) -> rrd_lsm::Result<(Rows, ProjectedReadEvidence)> {
    let mut stream = database.begin_projected_read(ProjectedReadRequest {
        ranges: vec![ProjectedReadRange::all()],
        projection,
        snapshot: database.snapshot(),
        budget,
    })?;
    let mut rows = Vec::new();
    while let Some(batch) = stream.next_batch()? {
        for index in 0..batch.len() {
            rows.push((
                batch.key(index).unwrap().to_vec(),
                batch.value(index).unwrap_or_default().to_vec(),
            ));
        }
    }
    Ok((rows, stream.evidence().clone()))
}

/// Every byte a page can hold for these rows, computed from what was written. Charging
/// each row for its whole row group comes to about thirty times this.
fn written_logical_bytes(rows: usize, row_groups: u64) -> u64 {
    let key_bytes = row(0).0.len() as u64;
    let rows = rows as u64;
    // One offset table for keys and one for values, each with a trailing entry.
    let offsets = 2 * 8 * (rows + row_groups);
    let sequences = 8 * rows;
    let validity = rows / 8 + row_groups;
    rows * (key_bytes + VALUE_BYTES as u64) + offsets + sequences + validity
}

fn assert_each_page_charged_once(evidence: &ProjectedReadEvidence, context: &str) {
    let groups = evidence.selected_row_groups;
    let charged = evidence.page_logical_bytes;
    assert!(
        charged <= written_logical_bytes(ROWS, groups),
        "{context}: charged {charged} logical bytes for {ROWS} rows in {groups} row groups",
    );
    assert!(
        charged >= (ROWS * VALUE_BYTES) as u64,
        "{context}: charged {charged} logical bytes, less than the values returned",
    );
    // The cache's scan handling relies on these touches, so each row still makes them.
    assert_eq!(evidence.key_offset_page_requests, groups, "{context}");
    assert_eq!(evidence.validity_page_requests, ROWS as u64, "{context}");
    assert_eq!(
        evidence.value_offset_page_requests, ROWS as u64,
        "{context}"
    );
    assert_eq!(evidence.value_data_page_requests, ROWS as u64, "{context}");
}

#[test]
fn a_scan_is_charged_once_for_each_page_it_visits() {
    let expected = (0..ROWS).map(row).collect::<Rows>();
    for physical in PHYSICAL {
        for page_cache_bytes in [0, 4 * 1024 * 1024] {
            let directory = tempfile::tempdir().unwrap();
            let mut database = Database::create_with_options(
                directory.path(),
                options(physical, page_cache_bytes),
            )
            .unwrap();
            write_run(&mut database, 0..ROWS);
            for max_batch_rows in [1, ProjectedReadBudget::default().max_batch_rows] {
                let context = format!("{physical} cache {page_cache_bytes} batch {max_batch_rows}");
                let (rows, evidence) = scan(
                    &database,
                    ProjectedReadProjection::KeyValue,
                    budget(max_batch_rows),
                )
                .unwrap();
                assert!(
                    rows == expected,
                    "{context}: rows differ from those written"
                );
                assert!(evidence.selected_row_groups > 1, "{context}");
                assert!(
                    (evidence.selected_row_groups as usize) < ROWS,
                    "{context}: rows must share row groups, or this proves nothing"
                );
                assert_each_page_charged_once(&evidence, &context);
            }
        }
    }
}

#[test]
fn a_key_scan_is_never_charged_for_value_pages() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        Database::create_with_options(directory.path(), options("bounded-raw", 0)).unwrap();
    write_run(&mut database, 0..ROWS);
    let (rows, evidence) = scan(&database, ProjectedReadProjection::Key, budget(64)).unwrap();
    assert_eq!(rows.len(), ROWS);
    assert!(evidence.selected_row_groups > 1);
    assert_eq!(evidence.value_offset_page_requests, 0);
    assert_eq!(evidence.value_data_page_requests, 0);
    assert!(
        evidence.page_logical_bytes < (ROWS * VALUE_BYTES) as u64,
        "charged {} logical bytes for a read of keys",
        evidence.page_logical_bytes,
    );
}

#[test]
fn interleaved_runs_are_charged_once_for_each_page() {
    let expected = (0..ROWS).map(row).collect::<Rows>();
    for max_batch_rows in [1, ProjectedReadBudget::default().max_batch_rows] {
        let directory = tempfile::tempdir().unwrap();
        let mut database =
            Database::create_with_options(directory.path(), options("bounded-raw", 0)).unwrap();
        // Alternating keys make the merge change run on every row, the hardest order
        // for a cursor to recognise a row group it has already paid for.
        write_run(&mut database, (0..ROWS).step_by(2));
        write_run(&mut database, (1..ROWS).step_by(2));
        let (rows, evidence) = scan(
            &database,
            ProjectedReadProjection::KeyValue,
            budget(max_batch_rows),
        )
        .unwrap();
        assert!(rows == expected, "batch {max_batch_rows}: rows differ");
        assert_eq!(evidence.selected_runs, 2);
        assert_each_page_charged_once(&evidence, &format!("two runs, batch {max_batch_rows}"));
    }
}

#[test]
fn superseded_versions_are_not_charged_for_their_value_pages() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        Database::create_with_options(directory.path(), options("bounded-raw", 0)).unwrap();
    write_run(&mut database, 0..ROWS);
    write_run(&mut database, 0..ROWS);
    let (rows, evidence) = scan(&database, ProjectedReadProjection::KeyValue, budget(64)).unwrap();
    assert_eq!(rows.len(), ROWS);
    assert_eq!(evidence.selected_runs, 2);
    assert_eq!(evidence.value_data_page_requests, ROWS as u64);
    // Charging the older run's value pages as well would reach twice the values alone.
    assert!(
        evidence.page_logical_bytes < 2 * (ROWS * VALUE_BYTES) as u64,
        "charged {} logical bytes",
        evidence.page_logical_bytes,
    );
}

#[test]
fn the_page_ceilings_still_refuse_one_unit_below_the_charge() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        Database::create_with_options(directory.path(), options("bounded-raw", 0)).unwrap();
    write_run(&mut database, 0..ROWS);
    let projection = ProjectedReadProjection::KeyValue;
    let observed = scan(&database, projection, budget(64)).unwrap().1;

    let exact = ProjectedReadBudget {
        max_page_logical_bytes: observed.page_logical_bytes,
        max_page_requests: observed.page_requests,
        ..budget(64)
    };
    assert_eq!(scan(&database, projection, exact).unwrap().0.len(), ROWS);

    let below_bytes = ProjectedReadBudget {
        max_page_logical_bytes: observed.page_logical_bytes - 1,
        ..budget(64)
    };
    assert!(matches!(
        scan(&database, projection, below_bytes),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PageLogicalBytes,
            ..
        })
    ));
    let below_requests = ProjectedReadBudget {
        max_page_requests: observed.page_requests - 1,
        ..budget(64)
    };
    assert!(matches!(
        scan(&database, projection, below_requests),
        Err(Error::ProjectedReadLimit {
            resource: ProjectedReadResource::PageRequests,
            ..
        })
    ));
}

#[test]
fn charging_once_does_not_change_what_a_scan_holds_or_releases() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        Database::create_with_options(directory.path(), options("bounded-raw", 0)).unwrap();
    write_run(&mut database, 0..ROWS);
    let memory = database.source_page_memory();

    let (_, evidence) = scan(&database, ProjectedReadProjection::KeyValue, budget(64)).unwrap();
    let total = written_logical_bytes(ROWS, evidence.selected_row_groups);
    assert_eq!(memory.stats().reserved_bytes, 0);
    assert!(
        memory.stats().peak_reserved_bytes < (total / 4) as usize,
        "a {total}-byte scan peaked at {} bytes of source pages",
        memory.stats().peak_reserved_bytes
    );

    let mut stream = database
        .begin_projected_read(ProjectedReadRequest {
            ranges: vec![ProjectedReadRange::all()],
            projection: ProjectedReadProjection::KeyValue,
            snapshot: database.snapshot(),
            budget: budget(4),
        })
        .unwrap();
    drop(stream.next_batch().unwrap().unwrap());
    stream.cancel().unwrap();
    assert_eq!(memory.stats().reserved_bytes, 0);
}
