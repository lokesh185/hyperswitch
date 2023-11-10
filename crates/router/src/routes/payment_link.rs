use actix_web::{web, Responder};
use router_env::{instrument, tracing, Flow};

use crate::{
    core::{api_locking, payment_link::*},
    services::{api, authentication as auth},
    AppState,
};

/// Payments Link - Retrieve
///
/// To retrieve the properties of a Payment Link. This may be used to get the status of a previously initiated payment or next action for an ongoing payment
#[utoipa::path(
    get,
    path = "/payment_link/{payment_link_id}",
    params(
        ("payment_link_id" = String, Path, description = "The identifier for payment link")
    ),
    request_body=RetrievePaymentLinkRequest,
    responses(
        (status = 200, description = "Gets details regarding payment link", body = RetrievePaymentLinkResponse),
        (status = 404, description = "No payment link found")
    ),
    tag = "Payments",
    operation_id = "Retrieve a Payment Link",
    security(("api_key" = []), ("publishable_key" = []))
)]
#[instrument(skip(state, req), fields(flow = ?Flow::PaymentLinkRetrieve))]

pub async fn payment_link_retrieve(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
    path: web::Path<String>,
    json_payload: web::Query<api_models::payments::RetrievePaymentLinkRequest>,
) -> impl Responder {
    let flow = Flow::PaymentLinkRetrieve;
    let payload = json_payload.into_inner();
    let (auth_type, _) = match auth::check_client_secret_and_get_auth(req.headers(), &payload) {
        Ok(auth) => auth,
        Err(err) => return api::log_and_return_error_response(error_stack::report!(err)),
    };
    api::server_wrap(
        flow,
        state,
        &req,
        payload.clone(),
        |state, _auth, _| retrieve_payment_link(state, path.clone()),
        &*auth_type,
        api_locking::LockAction::NotApplicable,
    )
    .await
}

pub async fn initiate_payment_link(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let flow = Flow::PaymentLinkInitiate;
    let (merchant_id, payment_id) = path.into_inner();
    let payload = api_models::payments::PaymentLinkInitiateRequest {
        payment_id,
        merchant_id: merchant_id.clone(),
    };
    api::server_wrap(
        flow,
        state,
        &req,
        payload.clone(),
        |state, auth, _| {
            intiate_payment_link_flow(
                state,
                auth.merchant_account,
                payload.merchant_id.clone(),
                payload.payment_id.clone(),
            )
        },
        &crate::services::authentication::MerchantIdAuth(merchant_id),
        api_locking::LockAction::NotApplicable,
    )
    .await
}

pub async fn payments_link_list(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
    payload: web::Query<api_models::payments::PaymentLinkListConstraints>,
) -> impl Responder {
    let flow = Flow::PaymentLinkList;
    let payload = payload.into_inner();
    api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, payload| list_payment_link(state, auth.merchant_account, payload),
        auth::auth_type(&auth::ApiKeyAuth, &auth::JWTAuth, req.headers()),
        api_locking::LockAction::NotApplicable,
    )
    .await
}
