#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{
	self as system,
	ensure_signed,
	ensure_none,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SendSignedTransaction,
		SignedPayload, SigningTypes, Signer, SubmitTransaction,
	}
};
use frame_support::{
	debug,
	dispatch::DispatchResult, decl_module, decl_storage, decl_event,
	traits::Get,
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	RuntimeDebug,
	offchain::{http, Duration, storage::StorageValueRef},
	traits::Zero,
	transaction_validity::{
		InvalidTransaction, ValidTransaction, TransactionValidity, TransactionSource,
		TransactionPriority,
	},
};
use codec::{Encode, Decode};
use sp_std::vec::Vec;
use lite_json::json::JsonValue;


pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");


pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
	};
	use sp_core::sr25519::Signature as Sr25519Signature;
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// This pallet's configuration trait
pub trait Trait: CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;

	// Configuration parameters

	/// A grace period after we send transaction.
	///
	/// To avoid sending too many transactions, we only attempt to send one
	/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
	/// sending between distinct runs of this offchain worker.
	type GracePeriod: Get<Self::BlockNumber>;

	/// Number of blocks of cooldown after unsigned transaction is included.
	///
	/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval` blocks.
	type UnsignedInterval: Get<Self::BlockNumber>;

	/// A configuration for base priority of unsigned transactions.
	///
	/// This is exposed so that it can be tuned for particular runtime, when
	/// multiple pallets send unsigned transactions.
	type UnsignedPriority: Get<TransactionPriority>;
}

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct PricePayload<Public, BlockNumber> {
	block_number: BlockNumber,
	price: u32,
	public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for PricePayload<T::Public, T::BlockNumber> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as ExampleOffchainWorker {
		/// A vector of recently submitted prices.
		///
		/// This is used to calculate average price, should have bounded size.
		Prices get(fn prices): Vec<u32>;
		/// Defines the block when next unsigned transaction will be accepted.
		///
		/// To prevent spam of unsigned (and unpayed!) transactions on the network,
		/// we only allow one transaction every `T::UnsignedInterval` blocks.
		/// This storage entry defines when new transaction is going to be accepted.
		NextUnsignedAt get(fn next_unsigned_at): T::BlockNumber;
	}
}

decl_event!(
	/// Events generated by the module.
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event generated when new price is accepted to contribute to the average.
		/// \[price, who\]
		NewPrice(u32, AccountId),
	}
);

decl_module! {
	/// A public part of the pallet.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		
		#[weight = 0]
		pub fn submit_price(origin, price: u32) -> DispatchResult {
			// Retrieve sender of the transaction.
			let who = ensure_signed(origin)?;
			// Add the price to the on-chain list.
			Self::add_price(who, price);
			Ok(())
		}

		
		#[weight = 0]
		pub fn submit_price_unsigned(origin, _block_number: T::BlockNumber, price: u32)
			-> DispatchResult
		{
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;
			// Add the price to the on-chain list, but mark it as coming from an empty address.
			Self::add_price(Default::default(), price);
			// now increment the block number at which we expect next unsigned transaction.
			let current_block = <system::Module<T>>::block_number();
			<NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
			Ok(())
		}

		#[weight = 0]
		pub fn submit_price_unsigned_with_signed_payload(
			origin,
			price_payload: PricePayload<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;
			// Add the price to the on-chain list, but mark it as coming from an empty address.
			Self::add_price(Default::default(), price_payload.price);
			// now increment the block number at which we expect next unsigned transaction.
			let current_block = <system::Module<T>>::block_number();
			<NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
			Ok(())
		}

		
		fn offchain_worker(block_number: T::BlockNumber) {
			
			debug::native::info!("Hello World from offchain workers!");

			
			let parent_hash = <system::Module<T>>::block_hash(block_number - 1u32.into());
			debug::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			
			let average: Option<u32> = Self::average_price();
			debug::debug!("Current price: {:?}", average);

			// For this example we are going to send both signed and unsigned transactions
			// depending on the block number.
			// Usually it's enough to choose one or the other.
			let should_send = Self::choose_transaction_type(block_number);
			let res = match should_send {
				TransactionType::Signed => Self::fetch_price_and_send_signed(),
				TransactionType::UnsignedForAny => Self::fetch_price_and_send_unsigned_for_any_account(block_number),
				TransactionType::UnsignedForAll => Self::fetch_price_and_send_unsigned_for_all_accounts(block_number),
				TransactionType::Raw => Self::fetch_price_and_send_raw_unsigned(block_number),
				TransactionType::None => Ok(()),
			};
			if let Err(e) = res {
				debug::error!("Error: {}", e);
			}
		}
	}
}

enum TransactionType {
	Signed,
	UnsignedForAny,
	UnsignedForAll,
	Raw,
	None,
}


impl<T: Trait> Module<T> {
	
	fn choose_transaction_type(block_number: T::BlockNumber) -> TransactionType {
		/// A friendlier name for the error that is going to be returned in case we are in the grace
		/// period.
		const RECENTLY_SENT: () = ();

		// Start off by creating a reference to Local Storage value.
		// Since the local storage is common for all offchain workers, it's a good practice
		// to prepend your entry with the module name.
		let val = StorageValueRef::persistent(b"example_ocw::last_send");
		
		let res = val.mutate(|last_send: Option<Option<T::BlockNumber>>| {
			// We match on the value decoded from the storage. The first `Option`
			// indicates if the value was present in the storage at all,
			// the second (inner) `Option` indicates if the value was succesfuly
			// decoded to expected type (`T::BlockNumber` in our case).
			match last_send {
				// If we already have a value in storage and the block number is recent enough
				// we avoid sending another transaction at this time.
				Some(Some(block)) if block_number < block + T::GracePeriod::get() => {
					Err(RECENTLY_SENT)
				},
				// In every other case we attempt to acquire the lock and send a transaction.
				_ => Ok(block_number)
			}
		});

		
		match res {
			// The value has been set correctly, which means we can safely send a transaction now.
			Ok(Ok(block_number)) => {
				
				let transaction_type = block_number % 3u32.into();
				if transaction_type == Zero::zero() { TransactionType::Signed }
				else if transaction_type == T::BlockNumber::from(1u32) { TransactionType::UnsignedForAny }
				else if transaction_type == T::BlockNumber::from(2u32) { TransactionType::UnsignedForAll }
				else { TransactionType::Raw }
			},
			// We are in the grace period, we should not send a transaction this time.
			Err(RECENTLY_SENT) => TransactionType::None,
			// We wanted to send a transaction, but failed to write the block number (acquire a
			// lock). This indicates that another offchain worker that was running concurrently
			// most likely executed the same logic and succeeded at writing to storage.
			// Thus we don't really want to send the transaction, knowing that the other run
			// already did.
			Ok(Err(_)) => TransactionType::None,
		}
	}

	/// A helper function to fetch the price and send signed transaction.
	fn fetch_price_and_send_signed() -> Result<(), &'static str> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC."
			)?
		}
		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

		// Using `send_signed_transaction` associated type we create and submit a transaction
		// representing the call, we've just created.
		// Submit signed will return a vector of results for all accounts that were found in the
		// local keystore with expected `KEY_TYPE`.
		let results = signer.send_signed_transaction(
			|_account| {
				// Received price is wrapped into a call to `submit_price` public function of this pallet.
				// This means that the transaction, when executed, will simply call that function passing
				// `price` as an argument.
				Call::submit_price(price)
			}
		);

		for (acc, res) in &results {
			match res {
				Ok(()) => debug::info!("[{:?}] Submitted price of {} cents", acc.id, price),
				Err(e) => debug::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	/// A helper function to fetch the price and send a raw unsigned transaction.
	fn fetch_price_and_send_raw_unsigned(block_number: T::BlockNumber) -> Result<(), &'static str> {
		// Make sure we don't fetch the price if unsigned transaction is going to be rejected
		// anyway.
		let next_unsigned_at = <NextUnsignedAt<T>>::get();
		if next_unsigned_at > block_number {
			return Err("Too early to send unsigned transaction")
		}

		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

		// Received price is wrapped into a call to `submit_price_unsigned` public function of this
		// pallet. This means that the transaction, when executed, will simply call that function
		// passing `price` as an argument.
		let call = Call::submit_price_unsigned(block_number, price);

		// Now let's create a transaction out of this call and submit it to the pool.
		// Here we showcase two ways to send an unsigned transaction / unsigned payload (raw)
		//
		// By default unsigned transactions are disallowed, so we need to whitelist this case
		// by writing `UnsignedValidator`. Note that it's EXTREMELY important to carefuly
		// implement unsigned validation logic, as any mistakes can lead to opening DoS or spam
		// attack vectors. See validation logic docs for more details.
		//
		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			.map_err(|()| "Unable to submit unsigned transaction.")?;

		Ok(())
	}

	/// A helper function to fetch the price, sign payload and send an unsigned transaction
	fn fetch_price_and_send_unsigned_for_any_account(block_number: T::BlockNumber) -> Result<(), &'static str> {
		// Make sure we don't fetch the price if unsigned transaction is going to be rejected
		// anyway.
		let next_unsigned_at = <NextUnsignedAt<T>>::get();
		if next_unsigned_at > block_number {
			return Err("Too early to send unsigned transaction")
		}

		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

		// -- Sign using any account
		let (_, result) = Signer::<T, T::AuthorityId>::any_account().send_unsigned_transaction(
			|account| PricePayload {
				price,
				block_number,
				public: account.public.clone()
			},
			|payload, signature| {
				Call::submit_price_unsigned_with_signed_payload(payload, signature)
			}
		).ok_or("No local accounts accounts available.")?;
		result.map_err(|()| "Unable to submit transaction")?;

		Ok(())
	}

	/// A helper function to fetch the price, sign payload and send an unsigned transaction
	fn fetch_price_and_send_unsigned_for_all_accounts(block_number: T::BlockNumber) -> Result<(), &'static str> {
		// Make sure we don't fetch the price if unsigned transaction is going to be rejected
		// anyway.
		let next_unsigned_at = <NextUnsignedAt<T>>::get();
		if next_unsigned_at > block_number {
			return Err("Too early to send unsigned transaction")
		}

		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let price = Self::fetch_price().map_err(|_| "Failed to fetch price")?;

		// -- Sign using all accounts
		let transaction_results = Signer::<T, T::AuthorityId>::all_accounts()
			.send_unsigned_transaction(
				|account| PricePayload {
					price,
					block_number,
					public: account.public.clone()
				},
				|payload, signature| {
					Call::submit_price_unsigned_with_signed_payload(payload, signature)
				}
			);
		for (_account_id, result) in transaction_results.into_iter() {
			if result.is_err() {
				return Err("Unable to submit transaction");
			}
		}

		Ok(())
	}

	/// Fetch current price and return the result in cents.
	fn fetch_price() -> Result<u32, http::Error> {
		// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
		// deadline to 2s to complete the external call.
		// You can also wait idefinitely for the response, however you may still get a timeout
		// coming from the host machine.
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
		// Initiate an external HTTP GET request.
		// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
		// you can find in `sp_io`. The API is trying to be similar to `reqwest`, but
		// since we are running in a custom WASM execution environment we can't simply
		// import the library here.
		let request = http::Request::get(
			"https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"
		);
		// We set the deadline for sending of the request, note that awaiting response can
		// have a separate deadline. Next we send the request, before that it's also possible
		// to alter request headers or stream body content in case of non-GET requests.
		let pending = request
			.deadline(deadline)
			.send()
			.map_err(|_| http::Error::IoError)?;

		// The request is already being processed by the host, we are free to do anything
		// else in the worker (we can send multiple concurrent requests too).
		// At some point however we probably want to check the response though,
		// so we can block current thread and wait for it to finish.
		// Note that since the request is being driven by the host, we don't have to wait
		// for the request to have it complete, we will just not read the response.
		let response = pending.try_wait(deadline)
			.map_err(|_| http::Error::DeadlineReached)??;
		// Let's check the status code before we proceed to reading the response.
		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		// Next we want to fully read the response body and collect it to a vector of bytes.
		// Note that the return object allows you to read the body in chunks as well
		// with a way to control the deadline.
		let body = response.body().collect::<Vec<u8>>();

		// Create a str slice from the body.
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			debug::warn!("No UTF8 body");
			http::Error::Unknown
		})?;

		let price = match Self::parse_price(body_str) {
			Some(price) => Ok(price),
			None => {
				debug::warn!("Unable to extract price from the response: {:?}", body_str);
				Err(http::Error::Unknown)
			}
		}?;

		debug::warn!("Got price: {} cents", price);

		Ok(price)
	}

	/// Parse the price from the given JSON string using `lite-json`.
	///
	/// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
	fn parse_price(price_str: &str) -> Option<u32> {
		let val = lite_json::parse_json(price_str);
		let price = val.ok().and_then(|v| match v {
			JsonValue::Object(obj) => {
				let mut chars = "USD".chars();
				obj.into_iter()
					.find(|(k, _)| k.iter().all(|k| Some(*k) == chars.next()))
					.and_then(|v| match v.1 {
						JsonValue::Number(number) => Some(number),
						_ => None,
					})
			},
			_ => None
		})?;

		let exp = price.fraction_length.checked_sub(2).unwrap_or(0);
		Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
	}

	/// Add new price to the list.
	fn add_price(who: T::AccountId, price: u32) {
		debug::info!("Adding to the average: {}", price);
		Prices::mutate(|prices| {
			const MAX_LEN: usize = 64;

			if prices.len() < MAX_LEN {
				prices.push(price);
			} else {
				prices[price as usize % MAX_LEN] = price;
			}
		});

		let average = Self::average_price()
			.expect("The average is not empty, because it was just mutated; qed");
		debug::info!("Current average price is: {}", average);
		// here we are raising the NewPrice event
		Self::deposit_event(RawEvent::NewPrice(price, who));
	}

	/// Calculate current average price.
	fn average_price() -> Option<u32> {
		let prices = Prices::get();
		if prices.is_empty() {
			None
		} else {
			Some(prices.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / prices.len() as u32)
		}
	}

	fn validate_transaction_parameters(
		block_number: &T::BlockNumber,
		new_price: &u32,
	) -> TransactionValidity {
		// Now let's check if the transaction has any chance to succeed.
		let next_unsigned_at = <NextUnsignedAt<T>>::get();
		if &next_unsigned_at > block_number {
			return InvalidTransaction::Stale.into();
		}
		// Let's make sure to reject transactions from the future.
		let current_block = <system::Module<T>>::block_number();
		if &current_block < block_number {
			return InvalidTransaction::Future.into();
		}

		// We prioritize transactions that are more far away from current average.
		//
		// Note this doesn't make much sense when building an actual oracle, but this example
		// is here mostly to show off offchain workers capabilities, not about building an
		// oracle.
		let avg_price = Self::average_price()
			.map(|price| if &price > new_price { price - new_price } else { new_price - price })
			.unwrap_or(0);

		ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
			// We set base priority to 2**20 and hope it's included before any other
			// transactions in the pool. Next we tweak the priority depending on how much
			// it differs from the current average. (the more it differs the more priority it
			// has).
			.priority(T::UnsignedPriority::get().saturating_add(avg_price as _))
			// This transaction does not require anything else to go before into the pool.
			// In theory we could require `previous_unsigned_at` transaction to go first,
			// but it's not necessary in our case.
			//.and_requires()
			// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
			// sure only one transaction produced after `next_unsigned_at` will ever
			// get to the transaction pool and will end up in the block.
			// We can still have multiple transactions compete for the same "spot",
			// and the one with higher priority will replace other one in the pool.
			.and_provides(next_unsigned_at)
			// The transaction is only valid for next 5 blocks. After that it's
			// going to be revalidated by the pool.
			.longevity(5)
			// It's fine to propagate that transaction to other peers, which means it can be
			// created even by nodes that don't produce blocks.
			// Note that sometimes it's better to keep it for yourself (if you are the block
			// producer), since for instance in some schemes others may copy your solution and
			// claim a reward.
			.propagate(true)
			.build()
	}
}

#[allow(deprecated)] // ValidateUnsigned
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	/// Validate unsigned call to this module.
	///
	/// By default unsigned transactions are disallowed, but implementing the validator
	/// here we make sure that some particular calls (the ones produced by offchain worker)
	/// are being whitelisted and marked as valid.
	fn validate_unsigned(
		_source: TransactionSource,
		call: &Self::Call,
	) -> TransactionValidity {
		// Firstly let's check that we call the right function.
		if let Call::submit_price_unsigned_with_signed_payload(
			ref payload, ref signature
		) = call {
			let signature_valid = SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
			if !signature_valid {
				return InvalidTransaction::BadProof.into();
			}
			Self::validate_transaction_parameters(&payload.block_number, &payload.price)
		} else if let Call::submit_price_unsigned(block_number, new_price) = call {
			Self::validate_transaction_parameters(block_number, new_price)
		} else {
			InvalidTransaction::Call.into()
		}
	}
}