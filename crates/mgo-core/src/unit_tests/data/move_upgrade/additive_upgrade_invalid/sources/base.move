// Copyright (c) MangoNet Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module base_addr::base {

    struct A<T> {
        f1: bool,
        f2: T
    }

    // new struct is fine
    struct B<T> {
        f2: bool,
        f1: T,
    }

    friend base_addr::friend_module;

    // new function is fine
    public fun return_1(): u64 { 1 }

    public fun return_0(): u64 { 0 }

    public fun plus_1(x: u64): u64 { x + 1 }

    public(friend) fun friend_fun(x: u64): u64 { x }

    // This is invalid since I just changed the code
    fun non_public_fun(y: bool): u64 { if (y) 0 else 2 }

    entry fun entry_fun() { }
}
