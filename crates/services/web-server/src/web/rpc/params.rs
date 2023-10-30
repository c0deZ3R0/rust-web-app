//! Base constructs for the typed RPC Params that will be used in their respective
//! rpc handler functions (e.g., `project_rpc::create_project` and `project_rpc::list_projects`).
//!
//! Most of these base constructs use generics for their respective data elements, allowing
//! each rpc handler function to receive the exact desired type.
//!
//! `IntoParams` or `IntoDefaultParams` are implemented to ensure these Params conform to the
//! `RpcRouter` (i.e., `rpc::router`) model.

use crate::web::rpc::router::{IntoDefaultParams, IntoParams};
use modql::filter::ListOptions;
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Params structure for any RPC Create call.
#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

impl<D> IntoParams for ParamsForCreate<D> where D: DeserializeOwned + Send {}

/// Params structure for any RPC Update call.
#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	pub id: i64,
	pub data: D,
}

impl<D> IntoParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}

/// Params structure for any RPC Update call.
#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}
impl IntoParams for ParamsIded {}

/// Params structure for any RPC List call.
#[derive(Deserialize, Default)]
pub struct ParamsList<F> {
	pub filter: Option<F>,
	pub list_options: Option<ListOptions>,
}

impl<D> IntoDefaultParams for ParamsList<D> where D: DeserializeOwned + Send + Default
{}
