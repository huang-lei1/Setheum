// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Common runtime code for Setheum, Neom and NewRome.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
		DispatchClass, Weight,
	},
};
use frame_system::limits;
pub use setheum_support::{ExchangeRate, Price, Rate, Ratio};
use primitives::{Balance, CurrencyId};
use sp_core::H160;
use sp_runtime::{
	traits::{Convert, Saturating},
	transaction_validity::TransactionPriority,
	FixedPointNumber, FixedPointOperand, Perbill,
};
use static_assertions::const_assert;

pub use primitives::currency::{
	GetDecimals, DNAR, JUSD, JEUR, JGBP, NEOM, JSAR, JCHF, JNGN, SDEX, HALAL,
};

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, primitives::Moment>;

// Priority of unsigned transactions
parameter_types! {
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

parameter_types! {
	pub FeeRateMatrix: [[Rate; 11]; 11] = [
		// when used_buffer_percent is 0%
		[
			Rate::zero(),
			Rate::saturating_from_rational(231487, 100000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(526013, 100000000), // 20%
			Rate::saturating_from_rational(106148, 10000000),  // 30%
			Rate::saturating_from_rational(243221, 10000000),  // 40%
			Rate::saturating_from_rational(597041, 10000000),  // 50%
			Rate::saturating_from_rational(126422, 1000000),   // 60%
			Rate::saturating_from_rational(214815, 1000000),   // 70%
			Rate::saturating_from_rational(311560, 1000000),   // 80%
			Rate::saturating_from_rational(410715, 1000000),   // 90%
			Rate::saturating_from_rational(510500, 1000000),   // 100%
		],
		// when used_buffer_percent is 10%
		[
			Rate::zero(),
			Rate::saturating_from_rational(260999, 100000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(584962, 100000000), // 20%
			Rate::saturating_from_rational(114942, 10000000),  // 30%
			Rate::saturating_from_rational(254703, 10000000),  // 40%
			Rate::saturating_from_rational(610531, 10000000),  // 50%
			Rate::saturating_from_rational(127866, 1000000),   // 60%
			Rate::saturating_from_rational(216285, 1000000),   // 70%
			Rate::saturating_from_rational(313035, 1000000),   // 80%
			Rate::saturating_from_rational(412191, 1000000),   // 90%
			Rate::saturating_from_rational(511976, 1000000),   // 100%
		],
		// when used_buffer_percent is 20%
		[
			Rate::zero(),
			Rate::saturating_from_rational(376267, 100000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(815202, 100000000), // 20%
			Rate::saturating_from_rational(149288, 10000000),  // 30%
			Rate::saturating_from_rational(299546, 10000000),  // 40%
			Rate::saturating_from_rational(663214, 10000000),  // 50%
			Rate::saturating_from_rational(133503, 1000000),   // 60%
			Rate::saturating_from_rational(222025, 1000000),   // 70%
			Rate::saturating_from_rational(318797, 1000000),   // 80%
			Rate::saturating_from_rational(417955, 1000000),   // 90%
			Rate::saturating_from_rational(517741, 1000000),   // 100%
		],
		// when used_buffer_percent is 30%
		[
			Rate::zero(),
			Rate::saturating_from_rational(807626, 100000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(167679, 10000000),  // 20%
			Rate::saturating_from_rational(277809, 10000000),  // 30%
			Rate::saturating_from_rational(467319, 10000000),  // 40%
			Rate::saturating_from_rational(860304, 10000000),  // 50%
			Rate::saturating_from_rational(154595, 1000000),   // 60%
			Rate::saturating_from_rational(243507, 1000000),   // 70%
			Rate::saturating_from_rational(340357, 1000000),   // 80%
			Rate::saturating_from_rational(439528, 1000000),   // 90%
			Rate::saturating_from_rational(539315, 1000000),   // 100%
		],
		// when used_buffer_percent is 40%
		[
			Rate::zero(),
			Rate::saturating_from_rational(219503, 10000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(444770, 10000000), // 20%
			Rate::saturating_from_rational(691029, 10000000), // 30%
			Rate::saturating_from_rational(100646, 1000000),  // 40%
			Rate::saturating_from_rational(149348, 1000000),  // 50%
			Rate::saturating_from_rational(222388, 1000000),  // 60%
			Rate::saturating_from_rational(312586, 1000000),  // 70%
			Rate::saturating_from_rational(409701, 1000000),  // 80%
			Rate::saturating_from_rational(508916, 1000000),  // 90%
			Rate::saturating_from_rational(608707, 1000000),  // 100%
		],
		// when used_buffer_percent is 50%
		[
			Rate::zero(),
			Rate::saturating_from_rational(511974, 10000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(102871, 1000000),  // 20%
			Rate::saturating_from_rational(156110, 1000000),  // 30%
			Rate::saturating_from_rational(213989, 1000000),  // 40%
			Rate::saturating_from_rational(282343, 1000000),  // 50%
			Rate::saturating_from_rational(364989, 1000000),  // 60%
			Rate::saturating_from_rational(458110, 1000000),  // 70%
			Rate::saturating_from_rational(555871, 1000000),  // 80%
			Rate::saturating_from_rational(655197, 1000000),  // 90%
			Rate::saturating_from_rational(755000, 1000000),  // 100%
		],
		// when used_buffer_percent is 60%
		[
			Rate::zero(),
			Rate::saturating_from_rational(804354, 10000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(161193, 1000000),  // 20%
			Rate::saturating_from_rational(242816, 1000000),  // 30%
			Rate::saturating_from_rational(326520, 1000000),  // 40%
			Rate::saturating_from_rational(414156, 1000000),  // 50%
			Rate::saturating_from_rational(506779, 1000000),  // 60%
			Rate::saturating_from_rational(603334, 1000000),  // 70%
			Rate::saturating_from_rational(701969, 1000000),  // 80%
			Rate::saturating_from_rational(801470, 1000000),  // 90%
			Rate::saturating_from_rational(901293, 1000000),  // 100%
		],
		// when used_buffer_percent is 70%
		[
			Rate::zero(),
			Rate::saturating_from_rational(942895, 10000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(188758, 1000000),  // 20%
			Rate::saturating_from_rational(283590, 1000000),  // 30%
			Rate::saturating_from_rational(379083, 1000000),  // 40%
			Rate::saturating_from_rational(475573, 1000000),  // 50%
			Rate::saturating_from_rational(573220, 1000000),  // 60%
			Rate::saturating_from_rational(671864, 1000000),  // 70%
			Rate::saturating_from_rational(771169, 1000000),  // 80%
			Rate::saturating_from_rational(870838, 1000000),  // 90%
			Rate::saturating_from_rational(970685, 1000000),  // 100%
		],
		// when used_buffer_percent is 80%
		[
			Rate::zero(),
			Rate::saturating_from_rational(985811, 10000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(197241, 1000000),  // 20%
			Rate::saturating_from_rational(296017, 1000000),  // 30%
			Rate::saturating_from_rational(394949, 1000000),  // 40%
			Rate::saturating_from_rational(494073, 1000000),  // 50%
			Rate::saturating_from_rational(593401, 1000000),  // 60%
			Rate::saturating_from_rational(692920, 1000000),  // 70%
			Rate::saturating_from_rational(792596, 1000000),  // 80%
			Rate::saturating_from_rational(892388, 1000000),  // 90%
			Rate::saturating_from_rational(992259, 1000000),  // 100%
		],
		// when used_buffer_percent is 90%
		[
			Rate::zero(),
			Rate::saturating_from_rational(997132, 10000000), // when demand_in_available_percent is 10%
			Rate::saturating_from_rational(199444, 1000000),  // 20%
			Rate::saturating_from_rational(299194, 1000000),  // 30%
			Rate::saturating_from_rational(398965, 1000000),  // 40%
			Rate::saturating_from_rational(498757, 1000000),  // 50%
			Rate::saturating_from_rational(598570, 1000000),  // 60%
			Rate::saturating_from_rational(698404, 1000000),  // 70%
			Rate::saturating_from_rational(798259, 1000000),  // 80%
			Rate::saturating_from_rational(898132, 1000000),  // 90%
			Rate::saturating_from_rational(998024, 1000000),  // 100%
		],
		// when used_buffer_percent is 100%
		[
			Rate::zero(),
			Rate::one(), // when demand_in_available_percent is 10%
			Rate::one(),  // 20%
			Rate::one(),  // 30%
			Rate::one(),  // 40%
			Rate::one(),  // 50%
			Rate::one(),  // 60%
			Rate::one(),  // 70%
			Rate::one(),  // 80%
			Rate::one(),  // 90%
			Rate::one(),  // 100%
		],
	];
}

// TODO: somehow estimate this value. Start from a conservative value.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// The ratio that `Normal` extrinsics should occupy. Start from a conservative value.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(70);
/// Parachain only have 0.5 second of computation time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 500 * WEIGHT_PER_MILLIS;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

parameter_types! {
	/// Maximum length of block. Up to 5MB.
	pub RuntimeBlockLength: limits::BlockLength =
		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub RuntimeBlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

parameter_types! {
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic
		.expect("Normal extrinsics have weight limit configured by default; qed")
		.saturating_sub(BlockExecutionWeight::get());
}

pub struct RelaychainValidatorFilter;
impl<AccountId> orml_traits::Contains<AccountId> for RelaychainValidatorFilter {
	fn contains(_: &AccountId) -> bool {
		true
	}
}

pub fn dollar(currency_id: CurrencyId) -> Balance {
	10u128.saturating_pow(currency_id.decimals())
}

pub fn cent(currency_id: CurrencyId) -> Balance {
	dollar(currency_id) / 100
}

pub fn millicent(currency_id: CurrencyId) -> Balance {
	cent(currency_id) / 1000
}

pub fn microcent(currency_id: CurrencyId) -> Balance {
	millicent(currency_id) / 1000
}

pub fn deposit(items: u32, bytes: u32, currency_id: CurrencyId) -> Balance {
	// TODO: come up with some value for this
	items as Balance * 15 * cent(currency_id) + (bytes as Balance) * 6 * cent(currency_id)
}
