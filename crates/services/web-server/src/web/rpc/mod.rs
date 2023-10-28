// region:    --- Modules

mod infra;
mod project_rpc;
mod task_rpc;

use crate::web::mw_auth::CtxW;
use crate::web::rpc::infra::{
	IntoHandlerParams, RpcHandler, RpcRouteTrait, RpcRouter,
};
use crate::web::rpc::project_rpc::{
	create_project, delete_project, list_projects, update_project,
};
use crate::web::rpc::task_rpc::{create_task, delete_task, list_tasks, update_task};
use crate::web::{Error, Result};
use axum::extract::{FromRef, State};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use modql::filter::ListOptions;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use std::sync::Arc;
use tracing::debug;

// endregion: --- Modules

// region:    --- RPC Types

/// JSON-RPC Request Body.
#[derive(Deserialize)]
struct RpcRequest {
	id: Option<Value>,
	method: String,
	params: Option<Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	data: D,
}

impl<D> IntoHandlerParams for ParamsForCreate<D> where D: DeserializeOwned + Send {}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	id: i64,
	data: D,
}

impl<D> IntoHandlerParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}

#[derive(Deserialize)]
pub struct ParamsIded {
	id: i64,
}

impl IntoHandlerParams for ParamsIded {}

#[derive(Deserialize)]
pub struct ParamsList<F> {
	filter: Option<F>,
	list_options: Option<ListOptions>,
}

impl<F> IntoHandlerParams for ParamsList<F> where F: DeserializeOwned + Send {}

impl<F> IntoHandlerParams for Option<ParamsList<F>>
where
	F: DeserializeOwned + Send,
{
	fn into_handler_params(value: Option<Value>) -> Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value)?),
			None => Ok(None),
		}
	}
}

// endregion: --- RPC Types

/// RPC basic information containing the id and method for additional logging purposes.
#[derive(Debug)]
pub struct RpcInfo {
	pub id: Option<Value>,
	pub method: String,
}

#[derive(Clone)]
struct RpcStates(ModelManager, Arc<RpcRouter>);

pub fn routes(mm: ModelManager) -> Router {
	// Build the combined RpcRouter.
	let mut rpc_router = RpcRouter::new()
		.append(task_rpc::rpc_router())
		.append(project_rpc::rpc_router());

	// Build the Axum States needed for this axum Router.
	let rpc_states = RpcStates(mm, Arc::new(rpc_router));

	// Build the Acum Router for '/rpc'
	Router::new()
		.route("/rpc", post(rpc_axum_handler))
		.with_state(rpc_states)
}

async fn rpc_axum_handler(
	State(RpcStates(mm, rpc_router)): State<RpcStates>,
	ctx: CtxW,
	Json(rpc_req): Json<RpcRequest>,
) -> Response {
	let ctx = ctx.0;

	// -- Create the RPC Info
	//    (will be set to the response.extensions)
	let rpc_info = RpcInfo {
		id: rpc_req.id.clone(),
		method: rpc_req.method.clone(),
	};

	// -- Exec Rpc Route
	let res = rpc_router
		.call(&rpc_info.method, ctx, mm, rpc_req.params)
		.await;

	// -- Build Rpc Success Response
	let res = res.map(|v| {
		let body_response = json!({
			"id": rpc_info.id,
			"result": v
		});
		Json(body_response)
	});

	// -- Create and Update Axum Response
	let mut res = res.into_response();
	res.extensions_mut().insert(rpc_info);

	res
}
