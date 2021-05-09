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

use setheum_primitives::{AccountId, TokenSymbol};
use hex_literal::hex;
use sc_chain_spec::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde_json::map::Map;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{FixedPointNumber, FixedU128, Perbill};

use crate::chain_spec::{
	get_account_id_from_seed, get_authority_keys_from_seed, Extensions, TELEMETRY_URL,
};

pub type ChainSpec = sc_service::GenericChainSpec<newrome_runtime::GenesisConfig, Extensions>;

fn newrome_session_keys(grandpa: GrandpaId, babe: BabeId) -> newrome_runtime::SessionKeys {
	newrome_runtime::SessionKeys { grandpa, babe }
}

/// Development testnet config (single validator Alice)
pub fn development_testnet_config() -> Result<ChainSpec, String> {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	TokenSymbol::get_info().iter().for_each(|(symbol_name, decimals)| {
		token_symbol.push(symbol_name.to_string());
		token_decimals.push(*decimals);
	});
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), token_decimals.into());

	let wasm_binary = newrome_runtime::WASM_BINARY.unwrap_or_default();

	Ok(ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![get_authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Default::default(),
	))
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	TokenSymbol::get_info().iter().for_each(|(symbol_name, decimals)| {
		token_symbol.push(symbol_name.to_string());
		token_decimals.push(*decimals);
	});
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), token_decimals.into());

	let wasm_binary = newrome_runtime::WASM_BINARY.ok_or("Dev runtime wasm binary not available")?;

	Ok(ChainSpec::from_genesis(
		"Local",
		"local",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				vec![
					get_authority_keys_from_seed("Alice"),
					get_authority_keys_from_seed("Bob"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Default::default(),
	))
}

pub fn latest_newrome_testnet_config() -> Result<ChainSpec, String> {
	let mut properties = Map::new();
	let mut token_symbol: Vec<String> = vec![];
	let mut token_decimals: Vec<u32> = vec![];
	TokenSymbol::get_info().iter().for_each(|(symbol_name, decimals)| {
		token_symbol.push(symbol_name.to_string());
		token_decimals.push(*decimals);
	});
	properties.insert("tokenSymbol".into(), token_symbol.into());
	properties.insert("tokenDecimals".into(), token_decimals.into());

	let wasm_binary = newrome_runtime::WASM_BINARY.ok_or("Newrome runtime wasm binary not available")?;

	Ok(ChainSpec::from_genesis(
		"Setheum Newrome TN1",
		"newrome1",
		ChainType::Live,
		//
		//TODO: Change `//setheum` to `//newrome`
		// SECRET="..."
		//
		// ROOT
		// ./target/debug/subkey inspect "$SECRET//setheum//root"
		//
		// ORACLE
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//oracle"
		//
		// VALIDATOR 1
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//1//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//1//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//1//grandpa"
		//
		// VALIDATOR 2
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//2//validator"
		// ./target/debug/subkey --sr25519 inspect "$SECRET//setheum//2//babe"
		// ./target/debug/subkey --ed25519 inspect "$SECRET//setheum//2//grandpa"
		//
		move || {
			newrome_genesis(
				wasm_binary,
				vec![
					(
						// 5CLg63YpPJNqcyWaYebk3LuuUVp3un7y1tmuV3prhdbnMA77
						hex!["0c2df85f943312fc853059336627d0b7a08669629ebd99b4debc6e58c1b35c2b"].into(),
						hex!["0c2df85f943312fc853059336627d0b7a08669629ebd99b4debc6e58c1b35c2b"].into(),
						hex!["21b5a771b99ef0f059c476502c018c4b817fb0e48858e95a238850d2b7828556"].unchecked_into(),
						hex!["948f15728a5fd66e36503c048cc7b448cb360a825240c48ff3f89efe050de608"].unchecked_into(),
					),
					(
						// 5FnLzAUmXeTZg5J9Ao5psKU68oA5PBekXqhrZCKDbhSCQi88
						hex!["a476c0050065dafac1e9ff7bf602fe628ceadacf67650f8317554bd571b73507"].into(),
						hex!["a476c0050065dafac1e9ff7bf602fe628ceadacf67650f8317554bd571b73507"].into(),
						hex!["77f3c27e98da7849ed0749e1dea449321a4a5a36a1dccf3f08fc0ab3af24c62e"].unchecked_into(),
						hex!["b4f5713322656d29930aa89efa5509554a36c40fb50a226eae0f38fc1a6ceb25"].unchecked_into(),
					),
					(
						// 5Gn5LuLuWNcY21Vue4QcFFD3hLvjQY3weMHXuEyejUbUnArt
						hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
						hex!["d07e538fee7c42be9b2627ea5caac9a30f1869d65af2a19df70138d5fcc34310"].into(),
						hex!["c5dfcf68ccf1a64ed4145383e4bbbb8bbcc50f654d87187c39df2b88a9683b7f"].unchecked_into(),
						hex!["4cc54799f38715771605a21e8272a7a1344667e4681611988a913412755a8a04"].unchecked_into(),
					),
				],
				// 5F98oWfz2r5rcRVnP9VCndg33DAAsky3iuoBSpaPUbgN9AJn
				hex!["8815a8024b06a5b4c8703418f52125c923f939a5c40a717f6ae3011ba7719019"].into(),
				vec![
					// 5F98oWfz2r5rcRVnP9VCndg33DAAsky3iuoBSpaPUbgN9AJn
					hex!["8815a8024b06a5b4c8703418f52125c923f939a5c40a717f6ae3011ba7719019"].into(),
					// 5Fe3jZRbKes6aeuQ6HkcTvQeNhkkRPTXBwmNkuAPoimGEv45
					hex!["9e22b64c980329ada2b46a783623bcf1f1d0418f6a2b5fbfb7fb68dbac5abf0f"].into(),
				],
			)
		},
		vec![
			// "/dns/testnet-bootnode-1.setheum.laminar.one/tcp/30333/p2p/12D3KooWAFUNUowRqCV4c5so58Q8iGpypVf3L5ak91WrHf7rPuKz"
			// 	.parse()
			// 	.unwrap(),
		],
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		Some("newrome1"),
		Some(properties),
		Default::default(),
	))
}

pub fn newrome_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../../../resources/newrome-dist.json")[..])
}

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> newrome_runtime::GenesisConfig {
	use newrome_runtime::{
		dollar, get_all_module_accounts, SetheumOracleConfig, AirDropConfig, BabeConfig, Balance, BalancesConfig,
		BandOracleConfig, DexConfig, EnabledTradingPairs,
		GeneralCouncilMembershipConfig, GrandpaConfig, MonetaryCouncilMembershipConfig, FinancialCouncilMembershipConfig, 
		IndicesConfig, NativeTokenExistentialDeposit, OperatorMembershipSetheumConfig, OperatorMembershipBandConfig,
		OrmlNFTConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
		SystemConfig, TechnicalCommitteeMembershipConfig, TokensConfig, TradingPair, VestingConfig, 
		DNAR, USDJ, EURJ, GBPJ, IDRJ, NGNJ, SETT, SDEX, DOT, BABE_GENESIS_EPOCH_CONFIG
	};
	#[cfg(feature = "std")]
	use sp_std::collections::btree_map::BTreeMap;

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let initial_balance: u128 = 1_000_000 * dollar(DNAR);
	let initial_staking: u128 = 100_000 * dollar(DNAR);

	// merge duplicated
	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + dollar(DNAR))) // bit more for fee
		.chain(endowed_accounts.iter().cloned().map(|k| (k, initial_balance)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	newrome_runtime::GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_indices: IndicesConfig { indices: vec![] },
		pallet_balances: BalancesConfig { balances },
		pallet_session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), newrome_session_keys(x.2.clone(), x.3.clone())))
				.collect::<Vec<_>>(),
		},
		pallet_staking: StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		pallet_sudo: SudoConfig { key: root_key.clone() },
		pallet_babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		pallet_grandpa: GrandpaConfig { authorities: vec![] },
		pallet_collective_Instance1: Default::default(),
		pallet_membership_Instance1: GeneralCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		//TODO: Add the Shura Council
		//TODO: Rename SerpCouncil to MonetaryCouncil
		pallet_collective_Instance2: Default::default(),
		pallet_membership_Instance2: MonetaryCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance3: Default::default(),
		pallet_membership_Instance3: FinancialCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance4: Default::default(),
		pallet_membership_Instance4: TechnicalCommitteeMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_membership_Instance5: OperatorMembershipSetheumConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_membership_Instance6: OperatorMembershipBandConfig {
			members: vec![root_key],
			phantom: Default::default(),
		},
		pallet_treasury: Default::default(),
		orml_tokens: TokensConfig {
			endowed_accounts: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), JUSD, 1_000_000 * dollar(JUSD)),
						(x.clone(), DOT, 1_000_000 * dollar(DOT)),
						(x.clone(), JGBP, 1_000_000 * dollar(JGBP)),
					]
				})
				.collect(),
		},
		
		module_airdrop: AirDropConfig {
			airdrop_accounts: vec![],
		},
		orml_oracle_Instance1: SetheumOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		orml_oracle_Instance2: BandOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		setheum_dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(TradingPair::new(JUSD, DOT), (1_000_000u128, 2_000_000u128)),
					(TradingPair::new(JUSD, JGBP), (1_000_000u128, 2_000_000u128)),
					(TradingPair::new(JUSD, DNAR), (1_000_000u128, 2_000_000u128)),
				],
			)],
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
	}
}

fn newrome_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> newrome_runtime::GenesisConfig {
	use newrome_runtime::{
		cent, dollar, get_all_module_accounts, SetheumOracleConfig, AirDropConfig, AirDropCurrencyId, BabeConfig,
		Balance, BalancesConfig, BandOracleConfig, DexConfig, 
		EnabledTradingPairs, GeneralCouncilMembershipConfig, GrandpaConfig, 
		MonetaryCouncilMembershipConfig, FinancialCouncilMembershipConfig, IndicesConfig, 
		NativeTokenExistentialDeposit, OperatorMembershipSetheumConfig, OperatorMembershipBandConfig, 
		OrmlNFTConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig, 
		TechnicalCommitteeMembershipConfig, TokensConfig, VestingConfig,
		DNAR, USDJ, EURJ, GBPJ, IDRJ, NGNJ, SETT, SDEX, DOT, BABE_GENESIS_EPOCH_CONFIG,
	};
	#[cfg(feature = "std")]
	use sp_std::collections::btree_map::BTreeMap;

	let existential_deposit = NativeTokenExistentialDeposit::get();

	let initial_balance: u128 = 1_000_000 * dollar(DNAR);
	let initial_staking: u128 = 100_000 * dollar(DNAR);

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), initial_staking + dollar(DNAR))) // bit more for fee
		.chain(endowed_accounts.iter().cloned().map(|k| (k, initial_balance)))
		.chain(
			get_all_module_accounts()
				.iter()
				.map(|x| (x.clone(), existential_deposit)),
		)
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	newrome_runtime::GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_indices: IndicesConfig { indices: vec![] },
		pallet_balances: BalancesConfig { balances },
		pallet_session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), newrome_session_keys(x.2.clone(), x.3.clone())))
				.collect::<Vec<_>>(),
		},
		pallet_staking: StakingConfig {
			validator_count: 5,
			minimum_validator_count: 1,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), initial_staking, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		pallet_sudo: SudoConfig { key: root_key.clone() },
		pallet_babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		pallet_grandpa: GrandpaConfig { authorities: vec![] },
		pallet_collective_Instance1: Default::default(),
		pallet_membership_Instance1: GeneralCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		//TODO: Add the Shura Council
		//TODO: Rename SerpCouncil to MonetaryCouncil
		pallet_collective_Instance2: Default::default(),
		pallet_membership_Instance2: MonetaryCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance3: Default::default(),
		pallet_membership_Instance3: FinancialCouncilMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_collective_Instance4: Default::default(),
		pallet_membership_Instance4: TechnicalCommitteeMembershipConfig {
			members: vec![root_key.clone()],
			phantom: Default::default(),
		},
		pallet_membership_Instance5: OperatorMembershipSetheumConfig {
			members: endowed_accounts.clone(),
			phantom: Default::default(),
		},
		pallet_membership_Instance6: OperatorMembershipBandConfig {
			members: endowed_accounts,
			phantom: Default::default(),
		},
		pallet_treasury: Default::default(),
		orml_tokens: TokensConfig {
			endowed_accounts: vec![
				(root_key.clone(), DOT, 1_000_000 * dollar(DOT)),
				(root_key, JGBP, 1_000_000 * dollar(JGBP)),
			],
		},
		module_airdrop: AirDropConfig {
			airdrop_accounts: {
				let dnar_airdrop_accounts_json =
					&include_bytes!("../../../../../resources/newrome-airdrop-DNAR.json")[..];
				let dnar_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(dnar_airdrop_accounts_json).unwrap();
				let neom_airdrop_accounts_json =
					&include_bytes!("../../../../../resources/newrome-airdrop-NEOM.json")[..];
				let neom_airdrop_accounts: Vec<(AccountId, Balance)> =
					serde_json::from_slice(neom_airdrop_accounts_json).unwrap();

				dnar_airdrop_accounts
					.iter()
					.map(|(account_id, dnar_amount)| (account_id.clone(), AirDropCurrencyId::DNAR, *dnar_amount))
					.chain(
						neom_airdrop_accounts
							.iter()
							.map(|(account_id, neom_amount)| (account_id.clone(), AirDropCurrencyId::NEOM, *neom_amount)),
					)
					.collect::<Vec<_>>()
			},
		},
		orml_oracle_Instance1: SetheumOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		orml_oracle_Instance2: BandOracleConfig {
			members: Default::default(), // initialized by OperatorMembership
			phantom: Default::default(),
		},
		setheum_dex: DexConfig {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		},
		orml_nft: OrmlNFTConfig {
			tokens: {
				let nft_airdrop_json = &include_bytes!("../../../../../resources/newrome-airdrop-NFT.json")[..];
				let nft_airdrop: Vec<(
					AccountId,
					Vec<u8>,
					module_nft::ClassData,
					Vec<(Vec<u8>, module_nft::TokenData, Vec<AccountId>)>,
				)> = serde_json::from_slice(nft_airdrop_json).unwrap();

				let mut tokens = vec![];
				for (class_owner, class_meta, class_data, nfts) in nft_airdrop {
					let mut tokens_of_class = vec![];
					for (token_meta, token_data, token_owners) in nfts {
						token_owners.iter().for_each(|account_id| {
							tokens_of_class.push((account_id.clone(), token_meta.clone(), token_data.clone()));
						});
					}

					tokens.push((
						class_owner.clone(),
						class_meta.clone(),
						class_data.clone(),
						tokens_of_class,
					));
				}

				tokens
			},
		},
	}
}
