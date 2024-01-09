use std::str::FromStr;

use astra::ResponseBuilder;
use hyper::{Response, StatusCode};
use maud::html;
use serde::{Deserialize, Serialize};

use crate::db::{Item, ItemData, ItemId};
use crate::fragment;
use crate::response::ResponseBuilderExt;
use crate::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemOrder {
    prev: Option<ItemId>,
    curr: ItemId,
    next: Option<ItemId>,
}

impl Service {
    pub fn home(
        &self,
        _req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        Ok(ResponseBuilder::new().body_html(self.home_page(None)?))
    }

    pub fn item_get(
        &self,
        _req: &mut astra::Request,
        params: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_id = ItemId::from_str(params.get("id").expect("id param not in the path params"))?;

        let item_data = self.load_item(item_id)?;

        Ok(ResponseBuilder::new().body_html(self.home_page(Some((item_id, item_data)))?))
    }
    pub fn item_order(
        &self,
        req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_order: ItemOrder = serde_urlencoded::from_reader(req.body_mut().reader())?;
        self.change_item_order(item_order.prev, item_order.curr, item_order.next)?;
        Ok(ResponseBuilder::new().body_static_bytes("foo", &[]))
    }

    pub fn item_update(
        &self,
        req: &mut astra::Request,
        params: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_id = ItemId::from_str(params.get("id").expect("id param not in the path params"))?;
        let item_data: ItemData = serde_urlencoded::from_reader(req.body_mut().reader())?;
        self.update_item(item_id, &item_data)?;
        Ok(ResponseBuilder::new().body_html(self.home_page(Some((item_id, item_data)))?))
    }

    pub fn item_create(
        &self,
        req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_data: ItemData = serde_urlencoded::from_reader(req.body_mut().reader())?;
        let item_id = self.create_item(item_data.clone())?;
        Ok(ResponseBuilder::new().body_html(html! {
            (Item::items_form("items", &self.read_items()?))
            (fragment::item_edit_form(Some((item_id, item_data)), Some("item-edit")))
        }))
    }

    pub fn item_edit(
        &self,
        _req: &mut astra::Request,
        params: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_id = ItemId::from_str(params.get("id").expect("id param not in the path params"))?;
        let item_data = self.load_item(item_id)?;
        Ok(ResponseBuilder::new()
            .body_html(fragment::item_edit_form(Some((item_id, item_data)), None)))
    }

    pub fn favicon_ico(
        &self,
        _req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        Ok(Response::builder()
            .cache_static()
            .body_static_bytes("image/gif", include_bytes!("../static/dpc.gif").as_slice()))
    }

    pub fn style_css(
        &self,
        _req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        Ok(Response::builder()
            // .cache_static()
            .body_static_str(
                "text/css",
                concat!(
                    include_str!("../static/style.css"),
                    include_str!("../static/style-htmx-send-error.css"),
                ),
            ))
    }

    pub fn script_js(
        &self,
        _req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        Ok(Response::builder()
            // .cache_static()
            .body_static_str(
                "application/javascript",
                concat!(
                    include_str!("../static/script.js"),
                    include_str!("../static/script-htmx-send-error.js"),
                ),
            ))
    }
}

pub fn not_found_404() -> astra::Response {
    Response::builder().status(404).body_html(fragment::page(
        "PAGE NOT FOUND",
        html! {
            h2 { "This page does not exist. Sorry!" }
            p {
                a href="/" { "Return to the main page" }
            }
        },
    ))
}

pub fn internal_error() -> astra::Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body_static_str("text/plain", "Internal Server Error")
}

pub fn too_many_requests_429() -> astra::Response {
    Response::builder()
        .cache_nostore()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body_static_str("text/plain", "Too Many Requests")
}
