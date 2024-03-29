// Copyright (c) MangoNet Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::*;
use mgo_types::base_types::ObjectRef as NativeObjectRef;

use super::{
    object::{Object, ObjectLookupKey},
    mgo_address::MgoAddress,
};

// A helper type representing the read of a specific version of an object. Intended to be
// "flattened" into other GraphQL types.
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct ObjectRead {
    pub native: NativeObjectRef,
    /// The checkpoint sequence number this was viewed at.
    pub checkpoint_viewed_at: u64,
}

#[Object]
impl ObjectRead {
    /// ID of the object being read.
    async fn address(&self) -> MgoAddress {
        self.address_impl()
    }

    /// Version of the object being read.
    async fn version(&self) -> u64 {
        self.version_impl()
    }

    /// 32-byte hash that identifies the object's contents at this version, encoded as a Base58
    /// string.
    async fn digest(&self) -> String {
        self.native.2.base58_encode()
    }

    /// The object at this version.  May not be available due to pruning.
    async fn object(&self, ctx: &Context<'_>) -> Result<Option<Object>> {
        Object::query(
            ctx.data_unchecked(),
            self.address_impl(),
            ObjectLookupKey::VersionAt {
                version: self.version_impl(),
                checkpoint_viewed_at: Some(self.checkpoint_viewed_at),
            },
        )
        .await
        .extend()
    }
}

impl ObjectRead {
    fn address_impl(&self) -> MgoAddress {
        MgoAddress::from(self.native.0)
    }

    fn version_impl(&self) -> u64 {
        self.native.1.value()
    }
}
