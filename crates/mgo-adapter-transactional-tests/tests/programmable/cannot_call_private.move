// Copyright (c) MangoNet Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// tests cannot call private with programmable transactions

//# init --addresses test=0x0 --accounts A

//# publish
module test::m1 {
    fun priv(_: &mut mgo::tx_context::TxContext) {}
}

//# programmable
//> 0: test::m1::priv();
