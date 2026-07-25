use crate::web::state::{CustomerRegistrationRequest, CustomerSession, WebState};
use crate::web::templates;
use axum::Form;
use axum::extract::State;
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

pub const CUSTOMER_SESSION_COOKIE: &str = "customer_session";
const CUSTOMER_SESSION_MAX_AGE_SECONDS: u32 = 8 * 60 * 60;

/// Shows the temporary customer registration page.
///
/// Customers register before entering the menu so checkout can use one
/// server-side session instead of repeating contact fields in the cart.
pub async fn start_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if current_customer_session(&state, &headers).is_some() {
        return Redirect::to("/").into_response();
    }
    Html(templates::customer_start_page(None, None)).into_response()
}

/// Creates a temporary dining-session record and stores only an opaque session
/// ID in an HTTP-only cookie.
pub async fn start_submit(
    State(state): State<WebState>,
    Form(payload): Form<CustomerRegistrationForm>,
) -> Response {
    println!("Customer registration request received.");
    match state.register_customer_session(payload.clone().into_request()) {
        Ok(session) => {
            println!("Customer validation succeeded; customer session created.");
            (
                StatusCode::SEE_OTHER,
                [
                    (LOCATION, "/"),
                    (
                        SET_COOKIE,
                        customer_session_cookie(&session.session_id).as_str(),
                    ),
                ],
            )
                .into_response()
        }
        Err(message) => {
            println!("Customer registration validation failed: {message}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Html(templates::customer_start_page(
                    Some(&payload),
                    Some(&message),
                )),
            )
                .into_response()
        }
    }
}

/// Renders the customer profile with identity, active orders, and session order
/// history. JavaScript refreshes order status every few seconds.
pub async fn profile_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let Some(session) = current_customer_session(&state, &headers) else {
        return Redirect::to("/start").into_response();
    };
    let view = state.menu_view();
    let orders = state
        .customer_orders_by_phone(&session.customer_phone)
        .unwrap_or_default();
    Html(templates::profile_page(&view, &session, &orders)).into_response()
}

/// Updates the profile details used for subsequent checkout orders.
pub async fn profile_submit(
    State(state): State<WebState>,
    headers: HeaderMap,
    Form(payload): Form<CustomerRegistrationForm>,
) -> Response {
    let Some(session_id) = customer_session_id_from_headers(&headers) else {
        return Redirect::to("/start").into_response();
    };
    match state.update_customer_session(&session_id, payload.into_request()) {
        Ok(_) => Redirect::to("/profile").into_response(),
        Err(message) => {
            let view = state.menu_view();
            if let Some(session) = state.customer_session(&session_id) {
                let orders = state
                    .customer_orders_by_phone(&session.customer_phone)
                    .unwrap_or_default();
                Html(templates::profile_page_with_message(
                    &view, &session, &orders, &message,
                ))
                .into_response()
            } else {
                Redirect::to("/start").into_response()
            }
        }
    }
}

/// Ends only the customer session. Admin login uses a different cookie and is
/// deliberately left untouched.
pub async fn end_session(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if let Some(session_id) = customer_session_id_from_headers(&headers) {
        state.clear_customer_session(&session_id);
    }
    (
        StatusCode::SEE_OTHER,
        [
            (LOCATION, "/start"),
            (SET_COOKIE, expired_customer_session_cookie().as_str()),
        ],
    )
        .into_response()
}

pub fn current_customer_session(state: &WebState, headers: &HeaderMap) -> Option<CustomerSession> {
    customer_session_id_from_headers(headers)
        .and_then(|session_id| state.customer_session(&session_id))
}

pub fn customer_session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    crate::web::session::cookie_value(headers, CUSTOMER_SESSION_COOKIE)
}

pub(crate) fn customer_session_cookie(session_id: &str) -> String {
    crate::web::session::session_cookie(
        CUSTOMER_SESSION_COOKIE,
        session_id,
        CUSTOMER_SESSION_MAX_AGE_SECONDS,
    )
}

fn expired_customer_session_cookie() -> String {
    crate::web::session::expired_session_cookie(CUSTOMER_SESSION_COOKIE)
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomerRegistrationForm {
    pub customer_name: String,
    pub customer_phone: String,
    pub table_number: String,
}

impl CustomerRegistrationForm {
    fn into_request(self) -> CustomerRegistrationRequest {
        CustomerRegistrationRequest {
            customer_name: self.customer_name,
            customer_phone: self.customer_phone,
            table_number: self.table_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::HeaderValue;
    use axum::http::header::COOKIE;

    fn valid_form() -> CustomerRegistrationForm {
        CustomerRegistrationForm {
            customer_name: "Aina".to_string(),
            customer_phone: "0123456789".to_string(),
            table_number: "T05".to_string(),
        }
    }

    async fn response_body(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        String::from_utf8(bytes.to_vec()).expect("response should be UTF-8")
    }

    #[tokio::test]
    async fn valid_registration_redirects_sets_cookie_and_unlocks_home() {
        let state = WebState::new(Vec::new(), Vec::new());
        let response = start_submit(State(state.clone()), Form(valid_form())).await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/");
        let set_cookie = response
            .headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(set_cookie.starts_with("customer_session="));
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Lax"));
        assert!(set_cookie.contains("Path=/"));
        assert!(set_cookie.contains("Max-Age="));
        assert!(!set_cookie.contains("Secure"));

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(set_cookie.split(';').next().unwrap()).unwrap(),
        );
        assert!(current_customer_session(&state, &headers).is_some());
        let home = crate::web::handlers::menu::customer_menu(State(state), headers).await;
        assert_eq!(home.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_registration_preserves_values_and_returns_specific_errors() {
        let state = WebState::new(Vec::new(), Vec::new());
        let mut form = valid_form();
        form.customer_phone = "invalid".to_string();
        let response = start_submit(State(state), Form(form)).await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response_body(response).await;
        assert!(body.contains("Aina"));
        assert!(body.contains("invalid"));
        assert!(body.contains("T05"));
        assert!(body.contains("Enter a valid Malaysian phone number."));
    }

    #[tokio::test]
    async fn missing_name_and_table_return_validation_errors() {
        let state = WebState::new(Vec::new(), Vec::new());
        let mut missing_name = valid_form();
        missing_name.customer_name.clear();
        let response = start_submit(State(state.clone()), Form(missing_name)).await;
        assert!(
            response_body(response)
                .await
                .contains("Customer name is required.")
        );

        let mut missing_table = valid_form();
        missing_table.table_number.clear();
        let response = start_submit(State(state), Form(missing_table)).await;
        assert!(
            response_body(response)
                .await
                .contains("Table number is required")
        );
    }

    #[tokio::test]
    async fn unregistered_home_redirects_to_start() {
        let response = crate::web::handlers::menu::customer_menu(
            State(WebState::new(Vec::new(), Vec::new())),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/start");
    }
}
