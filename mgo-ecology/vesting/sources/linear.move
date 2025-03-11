// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Description:
// This module defines a vesting strategy that allows users to claim coins linearly over time.

// Functionality:
// - Defines a linear vesting schedule.
// ===========================================================================================
#[allow(unused_const)]
module vesting::linear {
    use mgo::object::{Self, UID};
    use mgo::balance::{Balance as MgoBalance};
    use mgo::coin::{Self, Coin};
    use mgo::tx_context::TxContext;
    use mgo::clock::Clock;

    #[ext(error)]
    const EInvalidStartTime: u64 = 1;

    public struct Wallet<phantom T> has key, store {
        id: UID,
        // Amount of coins remaining in the wallet
        wallet_balance: MgoBalance<T>, // Renamed the field
        start: u64,
        claimed: u64,
        duration: u64
    }

    // Public Functions

    // Create a new wallet with the given coins and vesting duration.
    // Note that full amount of coins is stored in the wallet when it is created;
    // it is just that the coins need to be claimed over time.

    // @aborts with `EInvalidStartTime` if the start time is not in the future.
    public fun new_wallet<T>(
        coins: Coin<T>,
        clock: &Clock,
        start: u64,
        duration: u64,
        ctx: &mut TxContext,
    ): Wallet<T> {
     if (start <= (clock.timestamp_ms() as u64)) {
        abort EInvalidStartTime
    };

        Wallet {
            id: object::new(ctx),
            wallet_balance: coins.into_balance(),
            start,
            claimed: 0,
            duration
        }
    }

    // Claim the coins that are available for claiming at the current time.
    public fun claim<T>(
        self: &mut Wallet<T>,
        clock: &Clock,
        ctx: &mut TxContext,
    ): Coin<T> {
        let claimable_amount = self.claimable(clock);
        self.claimed = self.claimed + claimable_amount;
        coin::from_balance(self.wallet_balance.split(claimable_amount), ctx)
    }

    // Calculate the amount of coins that can be claimed at the current time.
    public fun claimable<T>(
        self: &Wallet<T>,
        clock: &Clock,
    ): u64 {
        let timestamp = clock.timestamp_ms();
        if (timestamp < self.start) return 0;
        if (timestamp >= self.start + self.duration) return self.wallet_balance.value();
        let elapsed = timestamp - self.start;
        let claimable: u128 = (self.wallet_balance.value() + self.claimed as u128) * (elapsed as u128) / (self.duration as u128);
        (claimable as u64) - self.claimed
    }

    // Delete the wallet if it is empty.
    public fun delete_wallet<T>(
        self: Wallet<T>,
    ) {
        let Wallet { id, start: _, wallet_balance, claimed: _, duration: _ } = self;
        id.delete();
        wallet_balance.destroy_zero();
    }

    // Accessors

    // Get the remaining balance of the wallet.
    public fun balance<T>(self: &Wallet<T>
    ): u64 {
        self.wallet_balance.value()
    }

    // Get the start time of the vesting schedule.
    public fun start<T>(self: &Wallet<T>
    ): u64 {
        self.start
    }

    // Get the duration of the vesting schedule.
    public fun duration<T>(self: &Wallet<T>
    ): u64 {
        self.duration
    }
}
