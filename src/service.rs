use std::net;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::format_err;
use db::{Item, ItemData, ItemId, ITEM_TABLE};
use hyper::http::HeaderValue;
use hyper::{header, Method};
use matchit::Match;
use rate_limit::{conventional, pre};
use redb::{ReadableTable, Table};
use resiter::Map;
use tracing::{debug, info, warn};

use crate::db::{Database, ItemValue, ITEM_ORDER_TABLE};
use crate::sortid::SortId;
use crate::{db, opts, rate_limit, routes};

type Router = matchit::Router<
    for<'a> fn(
        &Service,
        &'a mut astra::Request,
        &'a matchit::Params,
    ) -> anyhow::Result<astra::Response>,
>;

type State = ();

#[derive(Clone)]
pub struct Service {
    opts: opts::Opts,
    _state: Arc<State>,
    db: Database,
    router_get: Router,
    router_post: Router,
    pre_rate_limiter: pre::FastPreRateLimiter,
    rate_limiter: conventional::RateLimiter,
}

impl Service {
    pub fn new(opts: opts::Opts) -> anyhow::Result<Self> {
        let mut router_get = Router::new();
        let mut router_post = Router::new();
        router_get.insert("/", Self::home)?;
        router_post.insert("/item", Self::item_create)?;
        router_post.insert("/item/order", Self::item_order)?;
        router_get.insert("/item/:id", Self::item_get)?;
        router_post.insert("/item/:id", Self::item_update)?;
        router_get.insert("/item/:id/edit", Self::item_edit)?;
        router_get.insert("/favicon.ico", Self::favicon_ico)?;
        router_get.insert("/style.css", Self::style_css)?;
        router_get.insert("/script.js", Self::script_js)?;

        Self {
            _state: Default::default(),
            db: Database::open(&opts.db)?,
            opts,
            router_get,
            router_post,
            pre_rate_limiter: pre::FastPreRateLimiter::new(20, 60),
            rate_limiter: conventional::RateLimiter::new(60, 60),
        }
        .init_tables()
    }

    fn route(&self, req: &mut astra::Request) -> astra::Response {
        if self.opts.debug_delay {
            std::thread::sleep(Duration::from_millis(500));
        }
        let path = req.uri().path().to_owned();
        let (handler, params) = match (match *req.method() {
            Method::GET => &self.router_get,
            Method::POST => &self.router_post,
            _ => return routes::not_found_404(),
        })
        .at(&path)
        {
            Ok(Match { value, params }) => {
                let params = params.to_owned();
                (value, params)
            }
            // Otherwise, return a 404
            Err(_) => return routes::not_found_404(),
        };

        (handler)(self, req, &params).unwrap_or_else(|error| {
            warn!(%error, "Route handler error");
            routes::internal_error()
        })
    }

    fn handle_session(
        &self,
        req: &mut astra::Request,
        f: impl FnOnce(&mut astra::Request) -> astra::Response,
    ) -> astra::Response {
        let mut session = None;
        for (k, v) in RequestExt(req).iter_cookies() {
            if k == "session" {
                session = Some(v.to_owned());
            }
        }
        let mut resp = f(req);

        if session.is_none() {
            resp.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_str("session=booo").expect("can't fail"),
            );
        }

        resp
    }

    fn handle_rate_limiting(
        &self,
        req: &mut astra::Request,
        info: &astra::ConnectionInfo,
        f: impl FnOnce(&mut astra::Request) -> astra::Response,
    ) -> (astra::Response, Option<net::SocketAddr>) {
        let peer_addr = info.peer_addr();
        let peer_ip = peer_addr
            .map(|s| s.ip())
            .unwrap_or(net::IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        (
            if self.pre_rate_limiter.rate_limit(peer_ip) && self.rate_limiter.rate_limit(peer_ip) {
                routes::too_many_requests_429()
            } else {
                f(req)
            },
            peer_addr,
        )
    }

    pub fn init_tables(self) -> anyhow::Result<Self> {
        self.db.write_with(|dbtx| {
            let _ = dbtx.open_table(ITEM_TABLE)?;
            let _ = dbtx.open_table(ITEM_ORDER_TABLE)?;
            Ok(())
        })?;

        Ok(self)
    }

    pub fn read_items(&self) -> anyhow::Result<Vec<Item>> {
        let mut items = self.db.read_with(|dbtx| {
            Ok(dbtx
                .open_table(ITEM_TABLE)?
                .iter()?
                .map_ok(|(k, v)| (k.value(), v.value()))
                .collect::<Result<Vec<_>, _>>()?)
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

    pub fn create_item(&self, item_data: ItemData) -> anyhow::Result<ItemId> {
        self.db.write_with(|dbtx| {
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
            Ok(item_id)
        })
    }

    pub fn change_item_order(
        &self,
        prev_id: Option<ItemId>,
        curr_id: ItemId,
        next_id: Option<ItemId>,
    ) -> anyhow::Result<()> {
        self.db.write_with(|dbtx| {
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
            Ok(())
        })
    }

    pub fn load_item(&self, item_id: ItemId) -> anyhow::Result<ItemData> {
        self.db.read_with(|dbtx| {
            let item_table = dbtx.open_table(ITEM_TABLE)?;
            let item = item_table
                .get(item_id)?
                .ok_or_else(|| format_err!("item not found"))?
                .value();

            Ok(item.data)
        })
    }

    pub(crate) fn update_item(&self, item_id: ItemId, item_data: &ItemData) -> anyhow::Result<()> {
        self.db.write_with(|dbtx| {
            let mut item_table = dbtx.open_table(ITEM_TABLE)?;

            let item = item_table
                .get(item_id)?
                .ok_or_else(|| format_err!("item not found"))?
                .value();

            item_table.insert(
                item_id,
                ItemValue {
                    sort_id: item.sort_id,
                    data: item_data.to_owned(),
                },
            )?;

            Ok(())
        })
    }
}

pub struct RequestExt<'a>(&'a hyper::Request<astra::Body>);

impl<'a> RequestExt<'a> {
    fn iter_cookies(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .headers()
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(';'))
            .map(|s| s.trim())
            .flat_map(|s| s.split_once('='))
    }
}

impl astra::Service for Service {
    fn call(
        &self,
        mut req: hyper::Request<astra::Body>,
        info: astra::ConnectionInfo,
    ) -> astra::Response {
        debug!(
            method = %req.method(),
            path = %req.uri(),
            "request received"
        );
        let (resp, peer_addr) = self.handle_rate_limiting(&mut req, &info, |req| {
            self.handle_session(req, |req| self.route(req))
        });

        use crate::util::DisplayOption;
        info!(
            status = %resp.status(),
            method = %req.method(),
            path = %req.uri(),
            peer = %DisplayOption(peer_addr),
            "request"
        );
        resp
    }
}
