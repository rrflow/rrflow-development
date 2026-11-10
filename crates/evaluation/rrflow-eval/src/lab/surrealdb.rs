//! SurrealDB over its public `/sql` endpoint, as probed on 3.3.5.
use super::adapter::{
    describe, required_credentials, split_by_size, wait_until, CollectionSpec, Http, PointBatch,
    SearchRequest, SystemAdapter,
};
use super::manifest::{IndexSpec, LabMetric, SystemBinding, WorkloadSpec};
use super::Result;
use serde_json::Value;
use std::fmt::Write as _;
use std::sync::OnceLock;
use std::time::Instant;

const NAMESPACE: &str = "lab";
const DATABASE: &str = "lab";
/// The server refuses a larger `/sql` body by default, so inserts split by size.
const MAX_SQL_BYTES: usize = 960 * 1024;

fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let bits = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        output.push(TABLE[(bits >> 18) as usize & 63] as char);
        output.push(TABLE[(bits >> 12) as usize & 63] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(bits >> 6) as usize & 63] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[bits as usize & 63] as char
        } else {
            '='
        });
    }
    output
}

fn vector_text(values: &[f32]) -> String {
    let mut text = String::with_capacity(values.len() * 12 + 2);
    text.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            text.push(',');
        }
        // Debug formatting prints the shortest text that round-trips the f32.
        let _ = write!(text, "{value:?}");
    }
    text.push(']');
    text
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Table {
    name: String,
    exact_distance: Option<&'static str>,
}

pub struct SurrealAdapter {
    http: Http,
    table: OnceLock<Table>,
}

impl SurrealAdapter {
    pub fn new(binding: &SystemBinding, workload: &WorkloadSpec) -> Result<Self> {
        let (user, secret) = required_credentials(binding)?;
        let token = base64(format!("{user}:{secret}").as_bytes());
        Ok(Self {
            http: Http::new(
                &binding.endpoint,
                workload,
                vec![
                    ("Authorization".into(), format!("Basic {token}")),
                    ("Accept".into(), "application/json".into()),
                    ("Surreal-NS".into(), NAMESPACE.into()),
                    ("Surreal-DB".into(), DATABASE.into()),
                ],
            ),
            table: OnceLock::new(),
        })
    }

    fn table(&self) -> Result<&Table> {
        self.table
            .get()
            .ok_or_else(|| "table is not prepared".into())
    }

    /// Runs statements, failing on any statement that did not report OK: the
    /// endpoint answers HTTP 200 even when a statement fails.
    fn sql(&self, statements: String) -> Result<Vec<Value>> {
        let (status, text) = self.http.raw(
            "POST",
            "/sql",
            Some(("text/plain", statements.into_bytes())),
        )?;
        if !(200..300).contains(&status) {
            return Err(format!(
                "/sql HTTP {status}: {}",
                &text[..text.len().min(300)]
            ));
        }
        let value: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
        let statements = value
            .as_array()
            .ok_or_else(|| format!("unexpected /sql response: {}", describe(&value)))?;
        statements
            .iter()
            .map(|statement| {
                if statement["status"] == "OK" {
                    Ok(statement["result"].clone())
                } else {
                    Err(format!("statement failed: {}", describe(statement)))
                }
            })
            .collect()
    }
}

impl SystemAdapter for SurrealAdapter {
    fn observed_version(&self) -> Result<String> {
        let (status, text) = self.http.raw("GET", "/version", None)?;
        if status == 200 {
            Ok(text.trim().to_string())
        } else {
            Err(format!("/version returned HTTP {status}"))
        }
    }

    fn prepare(&self, spec: &CollectionSpec<'_>) -> Result<()> {
        let distance = match spec.metric {
            LabMetric::Cosine => "COSINE",
            LabMetric::Euclidean => "EUCLIDEAN",
            LabMetric::Dot => return Err("SurrealDB HNSW has no dot-product distance".into()),
        };
        let name = spec.name.replace('-', "_");
        let exact_distance = matches!(spec.index, IndexSpec::Exact).then_some(distance);
        let _ = self.table.set(Table {
            name: name.clone(),
            exact_distance,
        });
        let mut statements = format!(
            "DEFINE NAMESPACE IF NOT EXISTS {NAMESPACE}; USE NS {NAMESPACE}; \
             DEFINE DATABASE IF NOT EXISTS {DATABASE}; USE DB {DATABASE}; \
             REMOVE TABLE IF EXISTS {name}; DEFINE TABLE {name} SCHEMALESS; \
             DEFINE INDEX {name}_category ON {name} FIELDS category;"
        );
        if let IndexSpec::Hnsw { m, ef_construction } = spec.index {
            let _ = write!(
                statements,
                " DEFINE INDEX {name}_v ON {name} FIELDS v HNSW DIMENSION {} DIST {distance} \
                 TYPE F32 EFC {ef_construction} M {m};",
                spec.dimensions
            );
        }
        self.sql(statements)?;
        Ok(())
    }

    fn ingest(&self, batch: &PointBatch<'_>) -> Result<()> {
        let name = &self.table()?.name;
        let rows: Vec<String> = (0..batch.ids.len())
            .map(|index| {
                format!(
                    "{{id:{},v:{},category:{}}}",
                    batch.ids[index],
                    vector_text(batch.vector(index)),
                    batch.categories[index]
                )
            })
            .collect();
        for group in split_by_size(
            rows.iter().map(|row| row.len() + 1),
            MAX_SQL_BYTES,
            usize::MAX,
        ) {
            self.sql(format!(
                "INSERT INTO {name} [{}] RETURN NONE;",
                rows[group].join(",")
            ))?;
        }
        Ok(())
    }

    fn finish_ingest(&self, deadline: Instant) -> Result<()> {
        let table = self.table()?;
        if table.exact_distance.is_some() {
            return Ok(());
        }
        let name = &table.name;
        wait_until(deadline, "SurrealDB indexing", || {
            let results = self.sql(format!("INFO FOR INDEX {name}_v ON {name};"))?;
            let building = &results[0]["building"];
            Ok(building["status"] == "ready" && building["pending"].as_u64().unwrap_or(1) == 0)
        })
    }

    fn search(&self, request: &SearchRequest<'_>) -> Result<Vec<u64>> {
        let table = self.table()?;
        let operator = match (table.exact_distance, request.setting.ef) {
            (Some(distance), _) => format!("<|{},{distance}|>", request.k),
            (None, Some(ef)) => format!("<|{},{ef}|>", request.k),
            (None, None) => return Err("HNSW search needs ef".into()),
        };
        let filter = request
            .category
            .map(|category| format!("category = {category} AND "))
            .unwrap_or_default();
        let results = self.sql(format!(
            "SELECT id FROM {} WHERE {filter}v {operator} {};",
            table.name,
            vector_text(request.vector)
        ))?;
        let rows = results
            .first()
            .and_then(Value::as_array)
            .ok_or("search returned no rows array")?;
        rows.iter()
            .map(|row| {
                row["id"]
                    .as_str()
                    .and_then(|id| id.rsplit(':').next())
                    .and_then(|id| id.parse().ok())
                    .ok_or_else(|| format!("invalid record id in {}", describe(row)))
            })
            .collect()
    }

    fn index_evidence(&self) -> Result<Option<String>> {
        let table = self.table()?;
        if table.exact_distance.is_some() {
            return Ok(Some("exact scan by request".into()));
        }
        let name = &table.name;
        let results = self.sql(format!("INFO FOR INDEX {name}_v ON {name};"))?;
        Ok(Some(format!("HNSW index state {}", results[0]["building"])))
    }

    fn teardown(&self) -> Result<()> {
        self.sql(format!("REMOVE TABLE IF EXISTS {};", self.table()?.name))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{base64, vector_text};

    #[test]
    fn encodes_basic_authentication_like_rfc_4648() {
        assert_eq!(base64(b"root:root"), "cm9vdDpyb290");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
    }

    #[test]
    fn writes_floats_that_round_trip() {
        assert_eq!(vector_text(&[1.0, -0.25]), "[1.0,-0.25]");
    }
}
