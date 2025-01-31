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

//! Unit tests for the non-fungible-token module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

fn free_balance(who: &AccountId) -> Balance {
	<Runtime as Config>::Currency::free_balance(who)
}

fn reserved_balance(who: &AccountId) -> Balance {
	<Runtime as Config>::Currency::reserved_balance(who)
}

fn class_id_account() -> AccountId {
	<Runtime as Config>::ModuleId::get().into_sub_account(CLASS_ID)
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Default::default()
		));
		let event = Event::setheum_nft(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
		);
	});
}

#[test]
fn create_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			SetheumNFT::create_class(
				Origin::signed(BOB),
				vec![1],
				Properties(ClassProperty::Transferable | ClassProperty::Burnable)
			),
			pallet_balances::Error::<Runtime, _>::InsufficientBalance
		);
	});
}

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		let event = Event::setheum_nft(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 2 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			2
		));
		let event = Event::setheum_nft(crate::Event::MintedToken(class_id_account(), BOB, CLASS_ID, 2));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get()
				+ 2 * <Runtime as Config>::CreateTokenDeposit::get()
				+ Proxy::deposit(1u32)
		);
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_noop!(
			SetheumNFT::mint(Origin::signed(ALICE), BOB, CLASS_ID_NOT_EXIST, vec![1], 2),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			SetheumNFT::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 0),
			Error::<Runtime>::InvalidQuantity
		);

		assert_noop!(
			SetheumNFT::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 2),
			Error::<Runtime>::NoPermission
		);

		orml_nft::NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
			*id = <Runtime as orml_nft::Config>::TokenId::max_value()
		});
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 2 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_noop!(
			SetheumNFT::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2),
			orml_nft::Error::<Runtime>::NoAvailableTokenId
		);
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 2 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			2
		));

		assert_ok!(SetheumNFT::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)));
		let event = Event::setheum_nft(crate::Event::TransferredToken(BOB, ALICE, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_ok!(SetheumNFT::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));
		let event = Event::setheum_nft(crate::Event::TransferredToken(ALICE, BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn transfer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			SetheumNFT::transfer(Origin::signed(BOB), ALICE, (CLASS_ID_NOT_EXIST, TOKEN_ID)),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			SetheumNFT::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
			Error::<Runtime>::TokenIdNotFound
		);
		assert_noop!(
			SetheumNFT::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NoPermission
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Default::default()
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			SetheumNFT::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NonTransferable
		);
	});
}

#[test]
fn burn_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_ok!(SetheumNFT::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		let event = Event::setheum_nft(crate::Event::BurnedToken(BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
		);
	});
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			SetheumNFT::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID_NOT_EXIST)),
			Error::<Runtime>::TokenIdNotFound
		);

		assert_noop!(
			SetheumNFT::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NoPermission
		);

		orml_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
			class_info.as_mut().unwrap().total_issuance = 0;
		});
		assert_noop!(
			SetheumNFT::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			orml_nft::Error::<Runtime>::NumOverflow
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Default::default()
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			SetheumNFT::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NonBurnable
		);
	});
}

#[test]
fn destroy_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		); // + 100
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_ok!(SetheumNFT::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		assert_ok!(SetheumNFT::destroy_class(
			Origin::signed(class_id_account()),
			CLASS_ID,
			BOB
		));
		let event = Event::setheum_nft(crate::Event::DestroyedClass(class_id_account(), CLASS_ID, BOB));
		assert_eq!(last_event(), event);

		assert_eq!(reserved_balance(&class_id_account()), 2);
		assert_eq!(free_balance(&ALICE), 99700 + 100 - 2);
		assert_eq!(free_balance(&BOB), 300);
	});
}

#[test]
fn destroy_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(SetheumNFT::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
				.is_ok(),
			true
		);
		assert_ok!(SetheumNFT::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			SetheumNFT::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST, BOB),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			SetheumNFT::destroy_class(Origin::signed(BOB), CLASS_ID, BOB),
			Error::<Runtime>::NoPermission
		);

		assert_noop!(
			SetheumNFT::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB),
			Error::<Runtime>::CannotDestroyClass
		);

		assert_ok!(SetheumNFT::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		assert_ok!(SetheumNFT::destroy_class(
			Origin::signed(class_id_account()),
			CLASS_ID,
			BOB
		));
	});
}
