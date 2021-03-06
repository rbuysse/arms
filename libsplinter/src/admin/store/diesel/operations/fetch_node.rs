// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Provides the "fetch node" operation for the `DieselAdminServiceStore`.

use diesel::prelude::*;

use super::AdminServiceStoreOperations;
use crate::admin::store::{
    diesel::{
        models::{CircuitMemberModel, NodeEndpointModel},
        schema::{circuit_member, node_endpoint},
    },
    error::AdminServiceStoreError,
    CircuitNode,
};

pub(in crate::admin::store::diesel) trait AdminServiceStoreFetchNodeOperation {
    fn fetch_node(&self, node_id: &str) -> Result<Option<CircuitNode>, AdminServiceStoreError>;
}

impl<'a, C> AdminServiceStoreFetchNodeOperation for AdminServiceStoreOperations<'a, C>
where
    C: diesel::Connection,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
{
    fn fetch_node(&self, node_id: &str) -> Result<Option<CircuitNode>, AdminServiceStoreError> {
        self.conn.transaction::<Option<CircuitNode>, _, _>(|| {
            // Retrieves the `circuit_member` entry with the matching `node_id`.
            let member: CircuitMemberModel = circuit_member::table
                .filter(circuit_member::node_id.eq(&node_id))
                .first::<CircuitMemberModel>(self.conn)
                .optional()
                .map_err(|err| AdminServiceStoreError::QueryError {
                    context: String::from("Diesel error occurred fetching Node"),
                    source: Box::new(err),
                })?
                .ok_or_else(|| {
                    AdminServiceStoreError::NotFoundError(String::from(
                        "CircuitNode does not exist in AdminServiceStore",
                    ))
                })?;
            // Collect all `node_endpoint` entries with the matching `node_id`.
            let endpoints: Vec<String> = node_endpoint::table
                .filter(node_endpoint::node_id.eq(&member.node_id))
                .load(self.conn)
                .map_err(|err| AdminServiceStoreError::QueryError {
                    context: String::from("Failed to load CircuitNode endpoints"),
                    source: Box::new(err),
                })?
                .into_iter()
                .map(|endpoint_model: NodeEndpointModel| endpoint_model.endpoint)
                .collect();
            Ok(Some(CircuitNode {
                id: member.node_id,
                endpoints,
            }))
        })
    }
}
