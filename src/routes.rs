use astra::ResponseBuilder;
use hyper::{Response, StatusCode};
use maud::html;
use serde::{Deserialize, Serialize};

use crate::db::{Item, ItemData, ItemId};
use crate::fragment;
use crate::fragment::ResponseBuilderExt;
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
        Ok(ResponseBuilder::new().body_html(fragment::page(
            "home",
            html! {
                div ."container flex" {
                    div ."shink p-1" {
                        (Item::items_form_markup("items", &self.read_items()?))
                    }
                    form #item-edit ."p-1" {
                    }
                }
            },
        )))
    }

    pub fn item_order(
        &self,
        req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_order: ItemOrder = serde_json::from_reader(req.body_mut().reader())?;
        self.change_item_order(item_order.prev, item_order.curr, item_order.next)?;
        Ok(ResponseBuilder::new().body_static_bytes("foo", &[]))
    }

    pub fn item_create(
        &self,
        req: &mut astra::Request,
        _: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_data: ItemData = serde_json::from_reader(req.body_mut().reader())?;
        self.create_item(item_data)?;
        Ok(ResponseBuilder::new().body_html(html! {
            (Item::items_form_markup("items", &self.read_items()?))
        }))
    }

    pub fn item_edit(
        &self,
        _req: &mut astra::Request,
        params: &matchit::Params,
    ) -> anyhow::Result<astra::Response> {
        let item_id: ItemId =
            serde_json::from_str(params.get("id").expect("id param not in the path params"))?;
        let item = self.load_item(item_id)?;
        Ok(ResponseBuilder::new().body_html(html! {
            form #item-edit ."p-1" {
                input type="text" name="title" placeholder="Title..." ."border rounded p-1 m-1 w-full" value=(item.title);
                textarea  name="body" placeholder="Body..."  ."border rounded p-1 m-1 w-full h-24" { (item.body) }
            }
        }))
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
            .body_static_str("text/css", concat!("")))
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

pub fn too_many_requests_429() -> astra::Response {
    Response::builder()
        .cache_nostore()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body_static_str("text/plain", "Too Many Requests")
}
