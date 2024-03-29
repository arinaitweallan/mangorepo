// Copyright (c) MangoNet Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::id::UID;
use serde::{Deserialize, Serialize};

/// Rust version of the Move mgo::versioned::Versioned type.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Versioned {
    pub id: UID,
    pub version: u64,
}
