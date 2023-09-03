use std::sync::atomic::Ordering;

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Form;
use hyper::{Response, StatusCode};
use maud::{html, Markup};
use serde::{Deserialize, Serialize};

use crate::db::{Item, ItemData, ItemId};
use crate::error::AppResult;
use crate::fragment::ResponseBuilderExt;
use crate::service::Service;
use crate::{fragment, service};

pub async fn count(State(service): State<service::Service>) -> Markup {
    let count = service.state.count.fetch_add(1, Ordering::Relaxed) + 1;

    html! {
        (count)
    }
}

pub async fn home(State(service): State<service::Service>) -> AppResult<Markup> {
    Ok(fragment::page(
        "home",
        html! {
            div ."container flex" {
                div ."shink p-1" {
                    (Item::items_form_markup("items", &service.read_items().await?))
                }
                form #item-edit ."p-1" {
                    // input type="text" name="title" placeholder="Title..." ."border rounded p-1 m-1 w-full" {}
                    // textarea  name="body" placeholder="Body..."  ."border rounded p-1 m-1 w-full h-24" {}
                    // input type="text" name="something" placeholder="Something..."  ."border rounded p-1 m-1 w-full" {}
                }
            }
        },
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemOrder {
    prev: Option<ItemId>,
    curr: ItemId,
    next: Option<ItemId>,
}

pub async fn item_order(
    State(service): State<Service>,
    Form(item_order): Form<ItemOrder>,
) -> AppResult<()> {
    service.change_item_order(item_order.prev, item_order.curr, item_order.next)?;
    Ok(())
}

pub async fn item_create(
    State(service): State<Service>,
    Form(item_data): Form<ItemData>,
) -> AppResult<Markup> {
    service.create_item(item_data).await?;
    Ok(html! {
        (Item::items_form_markup("items", &service.read_items().await?))
    })
}

pub async fn item_edit(
    State(service): State<Service>,
    Path(item_id): Path<ItemId>,
) -> AppResult<Markup> {
    let item = service.load_item(item_id).await?;
    Ok(html! {
        form #item-edit ."p-1" {
            input type="text" name="title" placeholder="Title..." ."border rounded p-1 m-1 w-full" value=(item.title);
            textarea  name="body" placeholder="Body..."  ."border rounded p-1 m-1 w-full h-24" { (item.body) }
        }
    })
}

pub async fn not_found_404() -> impl IntoResponse {
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

pub async fn too_many_requests_429() -> impl IntoResponse {
    Response::builder()
        .cache_nostore()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body_static_str("text/plain", "Too Many Requests")
}

pub async fn favicon_ico() -> impl IntoResponse {
    Response::builder()
        .cache_static()
        .body_static_bytes("image/gif", include_bytes!("../static/dpc.gif").as_slice())
}

pub async fn style_css() -> impl IntoResponse {
    Response::builder()
        // .cache_static()
        .body_static_str("text/css", concat!(""))
}

pub async fn script_js() -> impl IntoResponse {
    Response::builder()
        // .cache_static()
        .body_static_str(
            "application/javascript",
            concat!(
                include_str!("../static/script.js"),
                include_str!("../static/script-htmx-send-error.js"),
            ),
        )
}
/// GET '/user/:id'
pub async fn get_user(Path(user_id): Path<u64>) -> Markup {
    html! { p { "User #"(user_id)  } }
}

pub async fn edit_post(Path(id): Path<String>) -> Markup {
    fragment::post_edit_form(&id, "Foo", "Content")
}

pub async fn save_post(Path(id): Path<String>) -> Markup {
    fragment::post(&id, "Foo", "Content")
}
