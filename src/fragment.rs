use maud::{html, Markup, DOCTYPE};

use crate::db::{Item, ItemData, ItemId};
use crate::service::Service;

pub fn page(title: &str, content: Markup) -> Markup {
    /// A basic header with a dynamic `page_title`.
    pub(crate) fn head(page_title: &str) -> Markup {
        html! {
            (DOCTYPE)
            html lang="en";
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="stylesheet" type="text/css" href="/style.css";
                title { "dpc - " (page_title) }
            }
        }
    }

    pub(crate) fn header() -> Markup {
        html! {
            header ."container py-5 flex flex-row place-content-center gap-6 items-center" {
                    div ."uppercase" { "Much Better Jira" }
                    ."" {
                        img src="/favicon.ico" style="image-rendering: pixelated;" alt="dpc's avatar image";
                    }
            }
        }
    }

    /// A static footer.
    pub(crate) fn footer() -> Markup {
        html! {
            script src="https://unpkg.com/htmx.org@1.9.4" {};
            script src="https://unpkg.com/sortablejs@1.15.0/Sortable.min.js" {};
            script type="module" src="/script.js" {};
        }
    }

    html! {
        (head(title))
        body ."container relative mx-auto !block" style="display: none" {
            div #"gray-out-page" ."fixed inset-0 send-error-hidden"  {
                div ."relative z-50 bg-white mx-auto max-w-sm p-10 flex flex-center flex-col gap-2" {
                    p { "Connection error" }
                    button ."rounded bg-red-700 text-white px-2 py-1" hx-get="/" hx-target="body" hx-swap="outerHTML" { "Reload" }
                }
                div ."inset-0 absolute z-0 bg-gray-500 opacity-50" {}
            }
            (header())

            main ."container" {
                (content)
            }
            (footer())
        }
    }
}

impl Service {
    pub fn home_page(&self, item: Option<(ItemId, ItemData)>) -> anyhow::Result<Markup> {
        Ok(page(
            "home",
            html! {
                div ."container flex flex-col md:flex-row" {
                    (item_edit_form(item, None))
                    div ."container shrink grow p-1" {
                        (Item::items_form("items", &self.read_items()?))
                    }
                }
            },
        ))
    }
}

impl Item {
    pub fn items_form(dom_id: &str, items: &[Item]) -> Markup {
        html! {
            div #(dom_id) ."sortable border-1 border-solid rounded-sm divide-y divide-solid shadow shadow-black"  hx-post="/item/order" hx-trigger="changed" hx-swap="none" {

                form ."items-new flex" hx-post="/item" hx-target="closest div" hx-swap="outerHTML" hx-indicator={"#item-edit, #"(dom_id)} {
                    input ."border shadow-inner shadow-gray-400 rounded m-1 p-1 rounded w-full" type="text" name="title" value="" placeholder="New..." autocomplete="off" {}
                    input ."hidden" type="submit" {}
                }

                @for item in items {
                    (item.items_sortable_row())
                }
            }
        }
    }

    pub fn items_sortable_row(&self) -> Markup {
        html! {
            div."draggable container p-1 even:bg-shade-02 group flex justify-between gap-1" #{ (self.id) } {
                div .handle { "<>" };
                div ."w-full" hx-trigger="click" hx-get={ "/item/" (self.id) } hx-push-url="true" hx-select="#item-edit" hx-target="#item-edit" hx-indicator="#item-edit" hx-swap="outerHTML" {
                     span ."group-hover:underline" {
                         (self.data.title)
                     }
                }
            }
        }
    }
}

pub fn item_edit_form(item: Option<(ItemId, ItemData)>, hx_swap_oob_id: Option<&str>) -> Markup {
    html! {
        @if let Some((item_id, item_data)) = item {
            form
                id=@if let Some(oob) = hx_swap_oob_id {
                    (oob)
                } @else {
                    "item-edit"
                }
                ."container p-1"
                hx-post={ "/item/" (item_id) }
                hx-trigger="submit, click from:find button, keydown[ctrlKey && keyCode==13]"
                hx-target="#items"
                hx-select="#items"
                hx-swap="outerHTML"
                hx-swap-oob=@if hx_swap_oob_id.is_some() { "outerHTML" }
            {
                input type="text" name="title" autofocus placeholder="Title..." ."border shadow-inner shadow-gray-400 rounded my-1 py-1 px-2 w-full" value=(item_data.title);
                textarea  name="body" placeholder="Body..."  ."border shadow-inner shadow-gray-400 rounded my-1 py-1 px-2 h-24 w-full" { (item_data.body) }
                button ."px-2 py-1 my-1 shadow-md bg-primary-btn text-custom-white rounded-md" { "Save" }
            }
        } @else {
            form #item-edit ."container hidden" {
            }
        }
    }
}
