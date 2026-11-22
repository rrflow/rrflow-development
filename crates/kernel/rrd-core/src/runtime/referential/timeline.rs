//! Sweep valid-time boundaries while retaining the newest knowledge version.
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct Interval<T> {
    pub from: u64,
    pub to: Option<u64>,
    pub value: T,
}

enum Event<T> {
    Start(u64, Option<T>),
    End(u64),
}

pub(super) struct Timeline<T> {
    events: BTreeMap<u64, Vec<Event<T>>>,
}

impl<T> Default for Timeline<T> {
    fn default() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }
}

impl<T: Clone> Timeline<T> {
    pub fn put(&mut self, from: u64, to: Option<u64>, cursor: u64, value: T) {
        self.events
            .entry(from)
            .or_default()
            .push(Event::Start(cursor, Some(value)));
        if let Some(end) = to {
            self.events.entry(end).or_default().push(Event::End(cursor));
        }
    }

    pub fn retire(&mut self, at: u64, cursor: u64) {
        self.events
            .entry(at)
            .or_default()
            .push(Event::Start(cursor, None));
    }

    pub fn finish(self) -> Vec<Interval<T>> {
        let mut intervals = Vec::new();
        let mut latest = 0;
        let mut value: Option<T> = None;
        let mut start = 0;
        for (at, events) in self.events {
            if start < at {
                if let Some(previous) = value.take() {
                    intervals.push(Interval {
                        from: start,
                        to: Some(at),
                        value: previous.clone(),
                    });
                    value = Some(previous);
                }
            }
            for event in events {
                match event {
                    Event::Start(cursor, next) if cursor > latest => {
                        latest = cursor;
                        value = next;
                    }
                    Event::End(cursor) if cursor == latest => value = None,
                    _ => {}
                }
            }
            start = at;
        }
        if let Some(value) = value {
            intervals.push(Interval {
                from: start,
                to: None,
                value,
            });
        }
        intervals
    }
}

/// First uncovered instant in a half-open interval. None means fully covered.
pub(super) fn first_gap(intervals: &[Interval<()>], from: u64, to: Option<u64>) -> Option<u64> {
    let begin = intervals.partition_point(|part| part.to.is_some_and(|end| end <= from));
    let mut covered = from;
    for part in &intervals[begin..] {
        if part.from > covered {
            return Some(covered);
        }
        match part.to {
            None => return None,
            Some(end) if to.is_some_and(|limit| end >= limit) => return None,
            Some(end) => covered = end,
        }
    }
    Some(covered)
}
