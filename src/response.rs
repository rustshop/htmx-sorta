use hyper::header;

pub trait ResponseBuilderExt {
    type Response;
    fn cache_static(self) -> Self;
    fn cache_nostore(self) -> Self;
    fn status_not_found(self) -> Self;

    fn body_html(self, html: maud::PreEscaped<String>) -> Self::Response;
    fn body_static_str(self, content_type: &str, content: &'static str) -> Self::Response;
    fn body_static_bytes(self, content_type: &str, content: &'static [u8]) -> Self::Response;
}

impl ResponseBuilderExt for astra::ResponseBuilder {
    type Response = astra::Response;
    fn cache_static(self) -> Self {
        self.header(
            header::CACHE_CONTROL,
            "max-age=86400, stale-while-revalidate=86400",
        )
    }
    fn cache_nostore(self) -> Self {
        self.header(header::CACHE_CONTROL, "nostore")
    }

    fn status_not_found(self) -> Self {
        self.status(hyper::StatusCode::NOT_FOUND)
    }

    fn body_html(self, html: maud::PreEscaped<String>) -> Self::Response {
        self.header(header::CONTENT_TYPE, "text/html")
            .body(astra::Body::new(html.into_string()))
            .unwrap()
    }

    fn body_static_str(self, content_type: &str, content: &'static str) -> Self::Response {
        self.header(header::CONTENT_TYPE, content_type)
            .body(astra::Body::new(content))
            .unwrap()
    }
    fn body_static_bytes(self, content_type: &str, content: &'static [u8]) -> Self::Response {
        self.header(header::CONTENT_TYPE, content_type)
            .body(astra::Body::new(content))
            .unwrap()
    }
}
