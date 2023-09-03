use axum::body::Full;
use axum::response::IntoResponse;
use hyper::http::response;
use hyper::{header, StatusCode};
use maud::{html, Markup, DOCTYPE};

use crate::db::Item;

pub fn page(title: &str, content: Markup) -> Markup {
    /// A basic header with a dynamic `page_title`.
    pub(crate) fn head(page_title: &str) -> Markup {
        html! {
            (DOCTYPE)
            html lang="en";
            head {
                meta charset="utf-8";
                script src="https://cdn.twind.style" crossorigin {}
                link rel="stylesheet" type="text/css" href="/style.css";
                // script src="https://cdn.jsdelivr.net/npm/@twind/core@1.1.3/core.global.js" crossorigin {}
                // script src="https://cdn.jsdelivr.net/npm/@twind/core@1" crossorigin {}
                title { "dpc - " (page_title) }
            }
        }
    }

    pub(crate) fn header() -> Markup {
        html! {
            header ."container flex mx-auto" {
                    nav ."column p-6" {
                        a href="/" ."p-6" { "Home" }
                        a href="/" ."p-6" { "Home2" }
                    }
                    ."p-6" {
                        img src="/favicon.ico" style="image-rendering: pixelated;" alt="dpc's avatar image";
                    }
            }
        }
    }

    /// A static footer.
    pub(crate) fn footer() -> Markup {
        html! {
            footer ."flex max-w-lg mx-auto justify-around" {
                div ."container" {
                    h3  {
                        "Dawid Ciężarkiewicz"
                        br;
                        span.subtitle { "aka " span.dpc { "dpc" } }
                    }
                }
                div ."container" {
                    p  {
                        "Copyleft "
                        a href="https://dpc.pw" { "dpc" }
                    }
                }
            }
            script src="https://unpkg.com/htmx.org@1.9.4" {};
            script src="https://unpkg.com/sortablejs@1.15.0/Sortable.min.js" {};
            script src="/script.js" {};
        }
    }

    html! {
        (head(title))
        body style="position: relative;"  {
            div #"gray-out-page" .hidden {
                button hx-get="/" hx-target="body" hx-swap="outerHTML" { "Reload" }
            }
            div #"htmx-send-error" .hidden {
                "Error sending the request"
            }
            (header())

            main ."container mx-auto" {
                (content)
            }
            (footer())
        }
    }
}

pub(crate) fn post(id: &str, title: &str, body: &str) -> Markup {
    html! {
        article .post #id {
            h2 { (title) }

            p {
                (body)
            }

            button hx-get={ "/post/"(id)"/edit" } hx-swap="outerHTML" hx-target={ "closest article" } { "Edit" }
        }
    }
}

pub(crate) fn post_edit_form(id: &str, title: &str, body: &str) -> Markup {
    html! {
        article .post #id {
            form {
                input ."border rounded p-2" type="text" value=(title);
                textarea wrap="soft" { (body) }
                button hx-post={ "/post/"(id) } hx-swap="outerHTML" hx-target={ "closest article" } { "Submit" }
            }
        }
    }
}

pub trait ResponseBuilderExt {
    type Response;
    fn cache_static(self) -> Self;
    fn cache_nostore(self) -> Self;
    fn status_not_found(self) -> Self;

    fn body_html(self, html: maud::PreEscaped<String>) -> Self::Response;
    fn body_static_str(self, content_type: &str, content: &'static str)
        -> axum::response::Response;
    fn body_static_bytes(
        self,
        content_type: &str,
        content: &'static [u8],
    ) -> axum::response::Response;
}

impl ResponseBuilderExt for response::Builder {
    type Response = axum::response::Response;
    fn cache_static(self) -> Self {
        self.header(
            "Cache-Control",
            "max-age=86400, stale-while-revalidate=86400",
        )
    }
    fn cache_nostore(self) -> Self {
        self.header("Cache-Control", "nostore")
    }

    fn status_not_found(self) -> Self {
        self.status(StatusCode::NOT_FOUND)
    }

    fn body_html(self, html: maud::PreEscaped<String>) -> Self::Response {
        self.header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(html.into_string())
            .unwrap()
            .into_response()
    }

    fn body_static_str(
        self,
        content_type: &str,
        content: &'static str,
    ) -> axum::response::Response {
        self.header(header::CONTENT_TYPE, content_type)
            .body(Full::from(content))
            .unwrap()
            .into_response()
    }
    fn body_static_bytes(
        self,
        content_type: &str,
        content: &'static [u8],
    ) -> axum::response::Response {
        self.header(header::CONTENT_TYPE, content_type)
            .body(Full::from(content))
            .unwrap()
            .into_response()
    }
}

impl Item {
    pub fn sortable_handle_markup(&self) -> Markup {
        html! {
            div."container p-1 even:bg-slate-50" #{ (self.id) } {
                 span.handle { "<>" } " ";
                 a href={ "/item/" (self.id) "/edit" } hx-trigger="click" hx-get={ "/item/" (self.id) "/edit" } hx-target="#item-edit" hx-swap="outerHTML" {
                     (self.data.title)
                 }
            }
        }
    }

    pub fn items_form_markup(dom_id: &str, items: &[Item]) -> Markup {
        html! {
            div #(dom_id) {
                form ."items-new flex" hx-post="/item" hx-target="closest div" hx-swap="outerHTML" {
                    input ."border rounded m-1 p-1 rounded w-full" type="text" name="title" value="" autocomplete="off" {}
                    input ."border rounded m-1 p-1 rounded w-full" type="text" name="body" value="" autocomplete="off" {}
                    input ."hidden" type="submit" {}
                }

                (Self::sortable_table_markup(items))
            }
        }
    }

    pub fn sortable_table_markup(items: &[Item]) -> Markup {
        html! {
            div hx-post="/item/order" hx-trigger="changed" hx-swap="none" {
                div ."htmx-indicator" {  "Updating..." }
                div ."sortable border-1 border-solid rounded-sm divide-y divide-solid" {
                    @for item in items {
                        (item.sortable_handle_markup())
                    }
                }
            }
        }
    }
}
