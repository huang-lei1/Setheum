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

// Disable the following lints
#![allow(clippy::borrowed_box)]

use crate::cli::{Cli, RelayChainCli, Subcommand};
use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use service::{chain_spec, IdentifyVariant};

pub use service::newrome_runtime::Block;

use log::info;
use polkadot_parachain::primitives::AccountIdConversion;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams, NetworkParams, Result,
	RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::Block as BlockT;
use std::{io::Write, net::SocketAddr};

#[cfg(feature = "with-setheum-runtime")]
const CHAIN_NAME: &str = "Setheum";
#[cfg(feature = "with-neom-runtime")]
const CHAIN_NAME: &str = "Neom";
#[cfg(feature = "with-newrome-runtime")]
const CHAIN_NAME: &str = "NewRome";

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		format!("{} Node", CHAIN_NAME)
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/Setheum-Labs/Setheum/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		let id = if id.is_empty() {
			// The binary prefix is always setheum.
			// Make NewRome the default chain spec.
			"newrome"
		} else {
			id
		};

		Ok(match id {
			#[cfg(feature = "with-newrome-runtime")]
			"dev" => Box::new(chain_spec::newrome::development_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"local" => Box::new(chain_spec::newrome::local_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"newrome" => Box::new(chain_spec::newrome::newrome_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"newrome-latest" => Box::new(chain_spec::newrome::latest_newrome_testnet_config()?),
			#[cfg(feature = "with-neom-runtime")]
			"neom" => Box::new(chain_spec::neom::neom_config()?),
			#[cfg(feature = "with-neom-runtime")]
			"neom-latest" => Box::new(chain_spec::neom::latest_neom_config()?),
			#[cfg(feature = "with-setheum-runtime")]
			"setheum" => Box::new(chain_spec::setheum::setheum_config()?),
			#[cfg(feature = "with-setheum-runtime")]
			"setheum-latest" => Box::new(chain_spec::setheum::latest_setheum_config()?),
			path => {
				let path = std::path::PathBuf::from(path);

				let starts_with = |prefix: &str| {
					path.file_name()
						.map(|f| f.to_str().map(|s| s.starts_with(&prefix)))
						.flatten()
						.unwrap_or(false)
				};

				if starts_with("neom") {
					#[cfg(feature = "with-neom-runtime")]
					{
						Box::new(chain_spec::neom::ChainSpec::from_json_file(path)?)
					}

					#[cfg(not(feature = "with-neom-runtime"))]
					return Err("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.".into());
				} else if starts_with("setheum") {
					#[cfg(feature = "with-setheum-runtime")]
					{
						Box::new(chain_spec::setheum::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-setheum-runtime"))]
					return Err("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.".into());
				} else {
					#[cfg(feature = "with-newrome-runtime")]
					{
						Box::new(chain_spec::newrome::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-newrome-runtime"))]
					return Err("NewRome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.".into());
				}
			}
		})
	}

	fn native_runtime_version(spec: &Box<dyn sc_service::ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_setheum() {
			#[cfg(feature = "with-setheum-runtime")]
			return &service::setheum_runtime::VERSION;
			#[cfg(not(feature = "with-setheum-runtime"))]
			panic!("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.");
		} else if spec.is_neom() {
			#[cfg(feature = "with-neom-runtime")]
			return &service::neom_runtime::VERSION;
			#[cfg(not(feature = "with-neom-runtime"))]
			panic!("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.");
		} else {
			#[cfg(feature = "with-newrome-runtime")]
			return &service::newrome_runtime::VERSION;
			#[cfg(not(feature = "with-newrome-runtime"))]
			panic!("NewRome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.");
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		format!("{} Parachain Collator", CHAIN_NAME)
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"{} parachain collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		rococo-collator [parachain-args] -- [relaychain-args]",
			CHAIN_NAME
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/Setheum-Labs/Setheum/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn set_default_ss58_version(spec: &Box<dyn service::ChainSpec>) {
	use sp_core::crypto::Ss58AddressFormat;

	let ss58_version = if spec.is_neom() {
		Ss58AddressFormat::NeomAccount
	} else if spec.is_setheum() {
		Ss58AddressFormat::setheumAccount
	} else {
		Ss58AddressFormat::SubstrateAccount
	};

	sp_core::crypto::set_default_ss58_version(ss58_version);
}

fn extract_genesis_wasm(chain_spec: &Box<dyn service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

/// Parses setheum specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Inspect(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.sync_run(|mut config| {
				let (client, _, _, _) = service::new_chain_ops(&mut config)?;
				cmd.run(client)
			})
		}

		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			#[cfg(feature = "with-setheum-runtime")]
			return runner.sync_run(|config| cmd.run::<service::setheum_runtime::Block, service::SetheumExecutor>(config));

			#[cfg(feature = "with-neom-runtime")]
			return runner
				.sync_run(|config| cmd.run::<service::neom_runtime::Block, service::NeomExecutor>(config));

			#[cfg(feature = "with-newrome-runtime")]
			return runner
				.sync_run(|config| cmd.run::<service::newrome_runtime::Block, service::NewRomeExecutor>(config));
		}

		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),

		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}

		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}

		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}

		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}

		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}

		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}

		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}

		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let block: Block = generate_genesis_block(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
			let raw_header = block.header().encode();
			let output_buf = if params.raw {
				raw_header
			} else {
				format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}

		Some(Subcommand::ExportGenesisWasm(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob = extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}

		None => {
			let runner = cli.create_runner(&*cli.run)?;

			let chain_spec = &runner.config().chain_spec;

			let is_newrome_dev = chain_spec.is_newrome_dev();

			set_default_ss58_version(chain_spec);

			runner.run_node_until_exit(|config| async move {
				// TODO
				let key = sp_core::Pair::generate().0;

				let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
				let relay_chain_id = extension.map(|e| e.relay_chain.clone());
				let para_id = extension.map(|e| e.para_id);

				let collator = cli.run.base.validator || cli.collator;

				if is_newrome_dev {
					return service::newrome_dev(config, cli.instant_sealing).map_err(Into::into);
				} else if cli.instant_sealing {
					return Err("Instant sealing can be turned on only in `--dev` mode".into());
				}

				let polkadot_cli = RelayChainCli::new(
					config.base_path.as_ref().map(|x| x.path().join("polkadot")),
					relay_chain_id,
					[RelayChainCli::executable_name()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let id = ParaId::from(cli.run.parachain_id.or(para_id).unwrap_or(666));

				let parachain_account = AccountIdConversion::<polkadot_primitives::v0::AccountId>::into_account(&id);

				let block: Block = generate_genesis_block(&config.chain_spec).map_err(|e| format!("{:?}", e))?;
				let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, config.task_executor.clone())
						.map_err(|err| format!("Relay chain argument error: {}", err))?;
				
				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Parachain genesis state: {}", genesis_state);
				info!("Is collating: {}", if collator { "yes" } else { "no" });

				// TODO: support Neom & setheum
				service::start_node::<service::newrome_runtime::RuntimeApi, service::NewRomeExecutor>(
					config,
					key,
					polkadot_config,
					id,
					collator,
				)
				.await
				.map(|r| r.0)
				.map_err(Into::into)
			})
		}
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(&self, default_listen_port: u16) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port)
	}

	fn init<C: SubstrateCli>(&self) -> Result<()> {
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() {
			self.chain_id.clone().unwrap_or_default()
		} else {
			chain_id
		})
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn telemetry_external_transport(&self) -> Result<Option<sc_service::config::ExtTransport>> {
		self.base.base.telemetry_external_transport()
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}
}
