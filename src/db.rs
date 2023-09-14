use std::fmt::Write as _;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{bail, Context};
use redb::{ReadTransaction, RedbKey, RedbValue, TableDefinition, WriteTransaction};
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
        f: impl FnOnce(&'_ WriteTransaction<'_>) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let mut dbtx = self.0.begin_write()?;

        let res = f(&mut dbtx)?;

        dbtx.commit()?;

        Ok(res)
    }

    pub fn read_with<R>(
        &self,
        f: impl FnOnce(&'_ ReadTransaction<'_>) -> anyhow::Result<R>,
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

pub const ITEM_TABLE: TableDefinition<ItemId, ItemValue> = TableDefinition::new("item");
pub const ITEM_ORDER_TABLE: TableDefinition<SortId, ItemId> = TableDefinition::new("item_order");

impl RedbKey for ItemId {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl RedbKey for SortId {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        SortId::cmp_raw(data1, data2)
    }
}

impl RedbValue for SortId {
    type SelfType<'a> = SortId;

    type AsBytes<'a> = &'a [u8];

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        SortId::from(data.to_vec())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.as_bytes()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("sort-id")
    }
}

impl RedbValue for ItemId {
    type SelfType<'a> = ItemId;

    type AsBytes<'a> = [u8; 8];

    fn fixed_width() -> Option<usize> {
        u64::fixed_width()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self(u64::from_bytes(data))
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        u64::as_bytes(&value.0)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("item-id")
    }
}

impl RedbValue for ItemValue {
    type SelfType<'a> = ItemValue;

    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::deserialize(data).expect("bincode deserialization error")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).expect("bincode serialization error")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("item-value")
    }
}
