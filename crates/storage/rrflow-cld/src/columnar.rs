//! Named nullable columns and the projected-read contract for CLD codecs.

use crate::{Error, Result};

/// One named column. Every value may be absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnarColumn {
    Utf8 {
        name: String,
        values: Vec<Option<String>>,
    },
    U64 {
        name: String,
        values: Vec<Option<u64>>,
    },
}

impl ColumnarColumn {
    pub fn name(&self) -> &str {
        match self {
            Self::Utf8 { name, .. } | Self::U64 { name, .. } => name,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Utf8 { values, .. } => values.len(),
            Self::U64 { values, .. } => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Columns of one length, with distinct names, in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnarTable {
    columns: Vec<ColumnarColumn>,
}

impl ColumnarTable {
    /// Refuses a table with no column, columns of different lengths, or a
    /// repeated or empty column name.
    pub fn new(columns: Vec<ColumnarColumn>) -> Result<Self> {
        let Some(first) = columns.first() else {
            return Err(Error::Execution(
                "a columnar table needs at least one column".into(),
            ));
        };
        let rows = first.len();
        for (index, column) in columns.iter().enumerate() {
            if column.name().is_empty() {
                return Err(Error::Execution("a column name is empty".into()));
            }
            if column.len() != rows {
                return Err(Error::Execution(format!(
                    "column {} has {} rows, the first column has {rows}",
                    column.name(),
                    column.len()
                )));
            }
            if columns[..index]
                .iter()
                .any(|earlier| earlier.name() == column.name())
            {
                return Err(Error::Execution(format!(
                    "column name {} is repeated",
                    column.name()
                )));
            }
        }
        Ok(Self { columns })
    }

    pub fn columns(&self) -> &[ColumnarColumn] {
        &self.columns
    }

    pub fn rows(&self) -> usize {
        self.columns[0].len()
    }

    pub fn column(&self, name: &str) -> Option<&ColumnarColumn> {
        self.columns.iter().find(|column| column.name() == name)
    }

    /// The named columns, in the order asked for. An unknown name is refused.
    pub fn project(&self, names: &[&str]) -> Result<Self> {
        let columns = names
            .iter()
            .map(|name| {
                self.column(name)
                    .cloned()
                    .ok_or_else(|| Error::Execution(format!("no column named {name}")))
            })
            .collect::<Result<Vec<_>>>()?;
        Self::new(columns)
    }
}

/// What a columnar format does with a table. A native rrflowCLD format and
/// every reference module meet this one contract, so they receive identical
/// tables and their answers and costs can be compared directly.
pub trait ColumnarFormat {
    /// A stable short name for results and messages.
    fn id(&self) -> &'static str;

    /// Encodes the whole table into one self-describing byte artifact.
    fn encode(&self, table: &ColumnarTable) -> Result<Vec<u8>>;

    /// Decodes every column, in stored order.
    fn decode(&self, artifact: &[u8]) -> Result<ColumnarTable>;

    /// Decodes only the named columns, in the order asked for. A format reads
    /// no other column where it can avoid it.
    fn decode_columns(&self, artifact: &[u8], names: &[&str]) -> Result<ColumnarTable>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf8(name: &str, values: &[Option<&str>]) -> ColumnarColumn {
        ColumnarColumn::Utf8 {
            name: name.into(),
            values: values
                .iter()
                .map(|value| value.map(str::to_owned))
                .collect(),
        }
    }

    #[test]
    fn a_table_refuses_ragged_repeated_empty_and_missing_columns() {
        assert!(ColumnarTable::new(Vec::new()).is_err());
        assert!(
            ColumnarTable::new(vec![utf8("a", &[Some("x")]), utf8("b", &[Some("x"), None])])
                .is_err()
        );
        assert!(ColumnarTable::new(vec![utf8("a", &[None]), utf8("a", &[None])]).is_err());
        assert!(ColumnarTable::new(vec![utf8("", &[None])]).is_err());

        let table = ColumnarTable::new(vec![
            utf8("a", &[Some("x"), None]),
            ColumnarColumn::U64 {
                name: "n".into(),
                values: vec![Some(1), None],
            },
        ])
        .unwrap();
        assert_eq!(table.rows(), 2);
        let projected = table.project(&["n", "a"]).unwrap();
        assert_eq!(projected.columns()[0].name(), "n");
        assert_eq!(projected.columns()[1].name(), "a");
        assert!(table.project(&["missing"]).is_err());
        assert!(table.project(&[]).is_err());
    }
}
