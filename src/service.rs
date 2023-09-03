use std::sync::Arc;

use anyhow::{format_err, Context};
use db::{Item, ItemData, ItemId, ITEM_TABLE};
use rate_limit::{conventional, pre};
use redb::{ReadableTable, Table};
use resiter::Map;

use super::State;
use crate::db::{ItemValue, ITEM_ORDER_TABLE};
use crate::sortid::SortId;
use crate::{db, opts, rate_limit};

#[derive(Clone)]
pub struct Service {
    pub(crate) state: Arc<State>,
    pub(crate) db: Arc<redb::Database>,
    // router: Router,
    pub(crate) pre_rate_limiter: pre::FastPreRateLimiter,
    pub(crate) rate_limiter: conventional::RateLimiter,
}

impl Service {
    pub async fn new(opts: &opts::Opts) -> anyhow::Result<Self> {
        let db = redb::Database::create(&opts.db)
            .with_context(|| format!("Failed to open database at {}", opts.db.display()))?;

        Self {
            state: Default::default(),
            db: db.into(),
            pre_rate_limiter: pre::FastPreRateLimiter::new(20, 60),
            rate_limiter: conventional::RateLimiter::new(10, 60),
        }
        .init_tables()
        .await
    }

    pub async fn init_tables(self) -> anyhow::Result<Self> {
        tokio::task::block_in_place(|| -> anyhow::Result<()> {
            let dbtx = self.db.begin_write()?;
            let _ = dbtx.open_table(ITEM_TABLE)?;
            let _ = dbtx.open_table(ITEM_ORDER_TABLE)?;
            dbtx.commit()?;
            Ok(())
        })?;

        Ok(self)
    }

    pub async fn read_items(&self) -> anyhow::Result<Vec<Item>> {
        let mut items = tokio::task::block_in_place(|| -> anyhow::Result<Vec<_>> {
            Ok(self
                .db
                .begin_read()?
                .open_table(ITEM_TABLE)?
                .iter()?
                .map_ok(|(k, v)| (k.value(), v.value()))
                .collect::<Result<_, _>>()?)
        })?;

        items.sort_unstable_by(|a, b| a.1.sort_id.cmp(&b.1.sort_id));

        Ok(items
            .into_iter()
            .map(|(k, v)| Item {
                id: k,
                data: v.data,
            })
            .collect())
    }

    pub fn get_last_item_id(
        &self,
        items_table: &Table<'_, '_, ItemId, ItemValue>,
    ) -> anyhow::Result<ItemId> {
        Ok(if let Some(res) = items_table.iter()?.next_back() {
            let res = res?;
            res.0.value()
        } else {
            ItemId(0)
        })
    }

    pub fn get_front_item_sort_id(
        &self,
        items_order_table: &Table<'_, '_, SortId, ItemId>,
    ) -> anyhow::Result<SortId> {
        let existing_first = if let Some(existing_first) = items_order_table.iter()?.next() {
            let existing_first = existing_first?;
            Some(existing_first.0.value())
        } else {
            None
        };

        Ok(SortId::in_front(existing_first.as_ref()))
    }

    pub async fn create_item(&self, item_data: ItemData) -> anyhow::Result<()> {
        tokio::task::block_in_place(|| -> anyhow::Result<()> {
            let dbtx = self.db.begin_write()?;
            {
                let mut item_order_table = dbtx.open_table(ITEM_ORDER_TABLE)?;
                let sort_id = self.get_front_item_sort_id(&item_order_table)?;

                let mut item_table = dbtx.open_table(ITEM_TABLE)?;
                let item_id = self.get_last_item_id(&item_table)?.increment();
                item_table.insert(
                    item_id,
                    ItemValue {
                        sort_id: sort_id.clone(),
                        data: item_data,
                    },
                )?;
                item_order_table.insert(sort_id, item_id)?;
            }
            dbtx.commit()?;
            Ok(())
        })
    }

    pub(crate) fn change_item_order(
        &self,
        prev_id: Option<ItemId>,
        curr_id: ItemId,
        next_id: Option<ItemId>,
    ) -> anyhow::Result<()> {
        tokio::task::block_in_place(|| -> anyhow::Result<()> {
            let dbtx = self.db.begin_write()?;
            {
                let mut item_table = dbtx.open_table(ITEM_TABLE)?;
                let curr = item_table
                    .get(curr_id)?
                    .ok_or_else(|| format_err!("curr_id element not found"))?
                    .value();

                let curr_old_sort_id = curr.sort_id.clone();
                let prev = if let Some(prev_id) = prev_id {
                    Some(
                        item_table
                            .get(prev_id)?
                            .ok_or_else(|| format_err!("prev_id element not found"))?
                            .value(),
                    )
                } else {
                    None
                };
                let next = if let Some(next_id) = next_id {
                    Some(
                        item_table
                            .get(next_id)?
                            .ok_or_else(|| format_err!("next_id element not found"))?
                            .value(),
                    )
                } else {
                    None
                };

                let curr_new_sort_id = match (
                    prev.as_ref().map(|p| &p.sort_id),
                    next.as_ref().map(|n| &n.sort_id),
                ) {
                    (Some(prev), Some(next)) => SortId::between(prev, next),
                    (Some(prev), None) => SortId::at_the_end(Some(prev)),
                    (None, Some(next)) => SortId::in_front(Some(next)),
                    (None, None) => {
                        /* nothing to do */
                        return Ok(());
                    }
                };

                if curr_new_sort_id != curr_old_sort_id {
                    let mut item_order_table = dbtx.open_table(ITEM_ORDER_TABLE)?;
                    item_table.insert(
                        curr_id,
                        ItemValue {
                            sort_id: curr_new_sort_id.clone(),
                            ..curr
                        },
                    )?;
                    item_order_table.remove(curr.sort_id)?;
                    item_order_table.insert(curr_new_sort_id, curr_id)?;
                }
            }
            dbtx.commit()?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn load_item(&self, item_id: ItemId) -> anyhow::Result<ItemData> {
        tokio::task::block_in_place(|| -> anyhow::Result<ItemData> {
            let dbtx = self.db.begin_read()?;
            {
                let item_table = dbtx.open_table(ITEM_TABLE)?;
                let item = item_table
                    .get(item_id)?
                    .ok_or_else(|| format_err!("item not found"))?
                    .value();

                Ok(item.data)
            }
        })
    }
}
