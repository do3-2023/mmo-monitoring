use askama::Template;
use axum::{
    http::StatusCode,
    response::Html,
    response::{IntoResponse, Response},
};
use tracing::event;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                event!(tracing::Level::ERROR, "error rendering template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error: {}", e),
                )
                    .into_response()
            }
        }
    }
}
