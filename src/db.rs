use std::fmt::Write as _;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{bail, Context};
use redb::{Key, ReadTransaction, TableDefinition, WriteTransaction};
use serde::{Deserialize, Serialize};

use crate::sortid::SortId;

#[derive(Clone)]
pub struct Database(Arc<redb::Database>);

impl From<redb::Database> for Database {
    fn from(db: redb::Database) -> Self {
        Self(Arc::new(db))
    }
}

impl Database {
    pub fn write_with<R>(
        &self,
        f: impl FnOnce(&'_ WriteTransaction) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let mut dbtx = self.0.begin_write()?;

        let res = f(&mut dbtx)?;

        dbtx.commit()?;

        Ok(res)
    }

    pub fn read_with<R>(
        &self,
        f: impl FnOnce(&'_ ReadTransaction) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let mut dbtx = self.0.begin_read()?;

        let res = f(&mut dbtx)?;

        Ok(res)
    }

    pub fn open(path: &PathBuf) -> anyhow::Result<Database> {
        Ok(Self::from(redb::Database::create(path).with_context(
            || format!("Failed to open database at {}", path.display()),
        )?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ItemId(pub u64);

impl AsRef<u64> for ItemId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl From<u64> for ItemId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl ItemId {
    pub(crate) fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl FromStr for ItemId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("i-") {
            bail!("does not start with 'i-'");
        }

        Ok(Self(
            s.split_at(2)
                .1
                .parse()
                .map_err(|_e| anyhow::format_err!("invalid number"))?,
        ))
    }
}

impl Serialize for ItemId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("i-{}", self.0))
    }
}

impl<'de> Deserialize<'de> for ItemId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        String::deserialize(deserializer)
            .and_then(|s| <ItemId as FromStr>::from_str(&s).map_err(Error::custom))
    }
}

impl maud::Render for ItemId {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str("i-");
        write!(buffer, "{}", self.0).expect("can't fail");
    }
}

#[derive(Debug)]
pub struct Item {
    pub id: ItemId,
    pub data: ItemData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemValue {
    pub sort_id: SortId,
    pub data: ItemData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemData {
    pub title: String,
    #[serde(default)]
    pub body: String,
}

pub const ITEM_TABLE: TableDefinition<ItemId, redb::AsBincode<ItemValue>> =
    TableDefinition::new("item");
pub const ITEM_ORDER_TABLE: TableDefinition<SortId, ItemId> = TableDefinition::new("item_order");

impl redb::AsRaw for ItemId {
    type Raw = u64;
}

impl Key for ItemId {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl redb::AsRaw for SortId {
    type Raw = Vec<u8>;
}

impl redb::Key for SortId {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}
impl redb::Name for SortId {
    const NAME: &'static str = "sort-id";
}
impl redb::Name for ItemId {
    const NAME: &'static str = "item-id";
}

impl redb::Name for ItemValue {
    const NAME: &'static str = "item-value";
}
