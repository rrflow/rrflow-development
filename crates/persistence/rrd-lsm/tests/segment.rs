use rrd_lsm::{Durability, Error, Memtable, Mutation, Segment, WalWriter, WriteBatch};
use std::io::{Seek, SeekFrom, Write};

fn table() -> Memtable {
    let directory = tempfile::tempdir().unwrap();
    let wal_path = directory.path().join("active.wal");
    let mut wal = WalWriter::create(&wal_path).unwrap();
    let first = WriteBatch::new(vec![
        Mutation::Put {
            key: b"alpha".to_vec(),
            value: b"one".to_vec(),
        },
        Mutation::Put {
            key: b"beta".to_vec(),
            value: b"two".to_vec(),
        },
    ])
    .unwrap();
    wal.append_write_batch(&first, Durability::Authoritative)
        .unwrap();
    let second = WriteBatch::new(vec![
        Mutation::Delete {
            key: b"alpha".to_vec(),
        },
        Mutation::Put {
            key: b"beta".to_vec(),
            value: b"three".to_vec(),
        },
    ])
    .unwrap();
    wal.append_write_batch(&second, Durability::Authoritative)
        .unwrap();
    drop(wal);
    let recovery = rrd_lsm::recover(&wal_path).unwrap();
    Memtable::recover(&recovery.batches).unwrap()
}

#[test]
fn immutable_segment_preserves_mvcc_reads_and_content_identity() {
    let directory = tempfile::tempdir().unwrap();
    let segments = directory.path().join("segments");
    let (segment, path) = Segment::write_from_memtable(&segments, &table()).unwrap();
    assert!(path.ends_with(format!("{}.seg", segment.descriptor.id)));
    assert_eq!(segment.descriptor.minimum_sequence, 1);
    assert_eq!(segment.descriptor.maximum_sequence, 4);
    assert_eq!(segment.descriptor.entries, 4);
    assert_eq!(segment.get(b"alpha", 1).unwrap(), Some(b"one".to_vec()));
    assert_eq!(segment.get(b"alpha", 3).unwrap(), None);
    assert_eq!(segment.get(b"beta", 2).unwrap(), Some(b"two".to_vec()));
    assert_eq!(segment.get(b"beta", 4).unwrap(), Some(b"three".to_vec()));

    let reopened = Segment::open(&path).unwrap();
    assert_eq!(reopened.descriptor, segment.descriptor);
    assert_eq!(
        reopened.scan(b"a", Some(b"z"), 2).unwrap(),
        vec![
            (b"alpha".to_vec(), b"one".to_vec()),
            (b"beta".to_vec(), b"two".to_vec()),
        ]
    );
    let (deduplicated, same_path) = Segment::write_from_memtable(&segments, &table()).unwrap();
    assert_eq!(same_path, path);
    assert_eq!(deduplicated.descriptor, segment.descriptor);
}

#[test]
fn v4_bytes_match_the_checked_in_format_vector() {
    let directory = tempfile::tempdir().unwrap();
    let (_, path) =
        Segment::write_from_memtable(&directory.path().join("segments"), &table()).unwrap();
    let bytes = std::fs::read(path).unwrap();
    let actual = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/segment-v4.hex");
    if std::env::var_os("RRFLOW_UPDATE_GOLDENS").is_some() {
        std::fs::write(fixture, format!("{actual}\n")).unwrap();
    }
    assert_eq!(
        format!("{actual}\n"),
        std::fs::read_to_string(fixture).unwrap()
    );
}

#[test]
fn corruption_and_truncation_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let segments = directory.path().join("segments");
    let (_, path) = Segment::write_from_memtable(&segments, &table()).unwrap();
    let original = std::fs::read(&path).unwrap();

    let corrupt = directory.path().join("corrupt.seg");
    std::fs::write(&corrupt, &original).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&corrupt)
        .unwrap();
    file.seek(SeekFrom::Start(50)).unwrap();
    file.write_all(&[original[50] ^ 0x80]).unwrap();
    file.sync_all().unwrap();
    assert!(matches!(
        Segment::open(&corrupt),
        Err(Error::InvalidSegment(_))
    ));

    let truncated = directory.path().join("truncated.seg");
    std::fs::write(&truncated, &original[..original.len() - 1]).unwrap();
    assert!(matches!(
        Segment::open(&truncated),
        Err(Error::InvalidSegment(_))
    ));
}

#[test]
fn sparse_segment_matches_the_memtable_for_point_range_and_mvcc_reads() {
    let directory = tempfile::tempdir().unwrap();
    let wal_path = directory.path().join("many.wal");
    let mut wal = WalWriter::create(&wal_path).unwrap();
    for phase in 0..3 {
        let operations = (0..200)
            .map(|index| {
                let key = format!("key:{index:04}").into_bytes();
                if phase == 2 && index % 5 == 0 {
                    Mutation::Delete { key }
                } else {
                    Mutation::Put {
                        key,
                        value: format!("value:{phase}:{index:04}|").repeat(8).into_bytes(),
                    }
                }
            })
            .collect();
        wal.append_write_batch(
            &WriteBatch::new(operations).unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    }
    drop(wal);
    let recovery = rrd_lsm::recover(&wal_path).unwrap();
    let table = Memtable::recover(&recovery.batches).unwrap();
    let (segment, path) =
        Segment::write_from_memtable(&directory.path().join("segments"), &table).unwrap();
    assert!(segment.row_group_count() >= 2);
    assert_eq!(&std::fs::read(&path).unwrap()[..8], b"RRDSEG04");
    assert_eq!(segment.descriptor.format_version, 4);
    assert_eq!(
        segment.descriptor.schema_digest,
        rrd_lsm::SEGMENT_SCHEMA_DIGEST
    );

    for snapshot in [1, 199, 200, 201, 399, 400, 401, 599, 600] {
        for index in 0..200 {
            let key = format!("key:{index:04}");
            assert_eq!(
                segment.get(key.as_bytes(), snapshot).unwrap().as_deref(),
                table.get(key.as_bytes(), snapshot),
                "point mismatch for {key} at sequence {snapshot}"
            );
        }
        for (start, end) in [
            (b"key:0000".as_slice(), Some(b"key:0200".as_slice())),
            (b"key:0063".as_slice(), Some(b"key:0131".as_slice())),
            (b"key:0190".as_slice(), None),
        ] {
            assert_eq!(
                segment.scan(start, end, snapshot).unwrap(),
                table.scan(start, end, snapshot),
                "range mismatch at sequence {snapshot}"
            );
        }
    }
}

fn rewrite_checksum(bytes: &mut Vec<u8>) {
    bytes.truncate(bytes.len() - 64);
    bytes.extend_from_slice(rrd_core::digest::sha256_hex(bytes).as_bytes());
}

#[test]
fn v4_rejects_authenticated_length_flags_and_page_corruption() {
    let directory = tempfile::tempdir().unwrap();
    let segments = directory.path().join("segments");
    let (_, path) = Segment::write_from_memtable(&segments, &table()).unwrap();
    let original = std::fs::read(path).unwrap();

    let mut wrong_length = original.clone();
    wrong_length[56..64].copy_from_slice(&999u64.to_le_bytes());
    rewrite_checksum(&mut wrong_length);
    let wrong_length_path = directory.path().join("wrong-length.seg");
    std::fs::write(&wrong_length_path, wrong_length).unwrap();
    assert!(matches!(
        Segment::open(&wrong_length_path),
        Err(Error::InvalidSegment(_))
    ));

    let mut unknown_flags = original.clone();
    unknown_flags[12..16].copy_from_slice(&2u32.to_le_bytes());
    rewrite_checksum(&mut unknown_flags);
    let unknown_flags_path = directory.path().join("unknown-flags.seg");
    std::fs::write(&unknown_flags_path, unknown_flags).unwrap();
    assert!(matches!(
        Segment::open(&unknown_flags_path),
        Err(Error::InvalidSegment(_))
    ));

    let mut corrupt_body = original;
    corrupt_body[256] ^= 0xff;
    rewrite_checksum(&mut corrupt_body);
    let corrupt_body_path = directory.path().join("corrupt-body.seg");
    std::fs::write(&corrupt_body_path, corrupt_body).unwrap();
    assert!(matches!(
        Segment::open(&corrupt_body_path),
        Err(Error::InvalidSegment(_))
    ));

    let mut corrupt_padding =
        std::fs::read(segments.read_dir().unwrap().next().unwrap().unwrap().path()).unwrap();
    corrupt_padding[296] = 1;
    rewrite_checksum(&mut corrupt_padding);
    let corrupt_padding_path = directory.path().join("corrupt-padding.seg");
    std::fs::write(&corrupt_padding_path, corrupt_padding).unwrap();
    assert!(matches!(
        Segment::open(&corrupt_padding_path),
        Err(Error::InvalidSegment(_))
    ));

    let mut impossible_entries =
        std::fs::read(segments.read_dir().unwrap().next().unwrap().unwrap().path()).unwrap();
    let index_offset = u64::from_le_bytes(impossible_entries[48..56].try_into().unwrap()) as usize;
    let first_key_bytes = u32::from_le_bytes(
        impossible_entries[index_offset + 44..index_offset + 48]
            .try_into()
            .unwrap(),
    ) as usize;
    let last_key_bytes = u32::from_le_bytes(
        impossible_entries[index_offset + 48..index_offset + 52]
            .try_into()
            .unwrap(),
    ) as usize;
    let first_page = index_offset + 32 + 32 + first_key_bytes + last_key_bytes;
    impossible_entries[first_page + 32..first_page + 40].copy_from_slice(&u64::MAX.to_le_bytes());
    rewrite_checksum(&mut impossible_entries);
    let impossible_entries_path = directory.path().join("impossible-entries.seg");
    std::fs::write(&impossible_entries_path, impossible_entries).unwrap();
    assert!(matches!(
        Segment::open(&impossible_entries_path),
        Err(Error::InvalidSegment(_))
    ));
}

fn pre_1_0_segment(version: u16) -> Vec<u8> {
    let magic = match version {
        1 => b"RRDSEG01".as_slice(),
        2 => b"RRDSEG02".as_slice(),
        3 => b"RRDSEG03".as_slice(),
        _ => unreachable!("test constructs only pre-1.0 formats"),
    };
    let mut bytes = magic.to_vec();
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes
}

#[test]
fn pre_1_0_and_unknown_segment_formats_are_rejected() {
    let directory = tempfile::tempdir().unwrap();
    for version in [1, 2, 3] {
        let path = directory.path().join(format!("format-{version}.seg"));
        std::fs::write(&path, pre_1_0_segment(version)).unwrap();
        assert!(matches!(
            Segment::open(&path),
            Err(Error::UnsupportedVersion {
                object: "segment",
                version: rejected
            }) if rejected == version
        ));
    }

    let segments = directory.path().join("segments");
    let (_, current) = Segment::write_from_memtable(&segments, &table()).unwrap();
    let mut bytes = std::fs::read(current).unwrap();
    bytes[8..10].copy_from_slice(&5u16.to_le_bytes());
    let unknown = directory.path().join("format-5.seg");
    std::fs::write(&unknown, bytes).unwrap();
    assert!(matches!(
        Segment::open(&unknown),
        Err(Error::UnsupportedVersion {
            object: "segment",
            version: 5
        })
    ));
}

#[test]
fn versions_remain_exact_when_one_key_exceeds_the_row_group_target() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = rrd_lsm::Database::create(directory.path()).unwrap();
    let operations = (0..160)
        .map(|version| Mutation::Put {
            key: b"one-key".to_vec(),
            value: vec![version as u8; 1024],
        })
        .collect();
    database
        .write_owned(
            WriteBatch::new(operations).unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();

    for sequence in [1, 63, 64, 65, 127, 128, 160] {
        assert_eq!(
            database
                .get(b"one-key", rrd_lsm::Snapshot { sequence })
                .unwrap(),
            Some(vec![(sequence - 1) as u8; 1024])
        );
    }
}

#[test]
fn authenticated_row_group_filters_skip_negative_point_read_io() {
    let directory = tempfile::tempdir().unwrap();
    let mut database = rrd_lsm::Database::create(directory.path()).unwrap();
    let operations = (0..200)
        .map(|index| Mutation::Put {
            key: format!("key:{:04}", index * 2).into_bytes(),
            value: vec![index as u8; 128],
        })
        .collect();
    database
        .write_owned(
            WriteBatch::new(operations).unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();
    let snapshot = database.snapshot();
    let before = database.page_cache_stats();

    let mut rejected = false;
    for index in 0..200 {
        let missing = format!("key:{:04}", index * 2 + 1);
        assert_eq!(database.get(missing.as_bytes(), snapshot).unwrap(), None);
        let after = database.page_cache_stats();
        if after.filter_negatives > before.filter_negatives {
            assert_eq!(after.loads, before.loads);
            rejected = true;
            break;
        }
    }
    assert!(
        rejected,
        "expected at least one deterministic negative filter hit"
    );

    assert_eq!(
        database.get(b"key:0000", snapshot).unwrap(),
        Some(vec![0; 128])
    );
    let after_present = database.page_cache_stats();
    assert!(after_present.filter_checks > before.filter_checks);
    assert!(after_present.loads > before.loads);
}

#[test]
fn an_open_v4_segment_detects_later_page_tampering_on_read() {
    let directory = tempfile::tempdir().unwrap();
    let segments = directory.path().join("segments");
    let (segment, path) = Segment::write_from_memtable(&segments, &table()).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    file.seek(SeekFrom::Start(256)).unwrap();
    file.write_all(&[original[256] ^ 0x40]).unwrap();
    file.sync_all().unwrap();

    assert!(matches!(
        segment.get(b"alpha", 1),
        Err(Error::InvalidSegment(_))
    ));
}

#[test]
fn database_page_cache_is_shared_bounded_and_observable() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        rrd_lsm::Database::create_with_page_cache(directory.path(), 4 * 1024).unwrap();
    let operations = (0..300)
        .map(|index| Mutation::Put {
            key: format!("cache:{index:04}").into_bytes(),
            value: vec![(index % 251) as u8; 1024],
        })
        .collect();
    database
        .write_owned(
            WriteBatch::new(operations).unwrap(),
            Durability::Authoritative,
        )
        .unwrap();
    database.flush_memtable(1).unwrap();
    let snapshot = database.snapshot();

    assert_eq!(database.page_cache_stats().entries, 0);
    database.get(b"cache:0001", snapshot).unwrap();
    let after_miss = database.page_cache_stats();
    assert_eq!(after_miss.misses, 6);
    assert_eq!(after_miss.loads, 6);
    assert!(after_miss.bytes_read > 0);
    assert!(after_miss.bytes_decoded >= after_miss.bytes_read);
    database.get(b"cache:0001", snapshot).unwrap();
    let after_hit = database.page_cache_stats();
    assert!(after_hit.hits >= 5);
    assert!(after_hit.loads <= after_miss.loads + 1);
    assert!(after_hit.bytes_read >= after_miss.bytes_read);
    for index in [70, 140, 210, 299] {
        database
            .get(format!("cache:{index:04}").as_bytes(), snapshot)
            .unwrap();
    }
    let final_stats = database.page_cache_stats();
    assert!(final_stats.resident_bytes <= final_stats.capacity_bytes);
    assert!(final_stats.evictions > 0);
}
