// Copyright (c) MangoNet Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module mgo_system::msim_extra_1 {
    use mgo::object::UID;
    use mgo::tx_context::TxContext;

    struct Type has drop {
        x: u64
    }

    struct Obj has key {
        id: UID,
    }

    struct AlmostObj {
        id: UID,
    }

    struct NewType {
        t: Type,
    }

    public fun canary(): u64 {
        private_function(20, 21)
    }

    entry fun mint(_ctx: &mut TxContext) {}

    public entry fun entry_fun() {}

    /// Bit of a confusing function name, but we're testing that a
    /// once private function can be made public.
    public fun private_function(x: u64, y: u64): u64 {
        x + y + 2
    }

    // Removing this function
    // fun private_function_2(x: u64): u64 { x }

    entry fun private_function_3(_x: u64) {}

    public fun generic<T: drop>(_t: T) {}
}

module mgo_system::msim_extra_2 {
    public fun bar(): u64 {
        43
    }
}
