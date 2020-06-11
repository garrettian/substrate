// This file is part of Substrate.

// Copyright (C) 2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Implementation of the `sign-transaction` subcommand
use sc_cli::{
	Error, CliConfiguration, KeystoreParams, SharedParams,
	pair_from_suri, decode_hex, with_crypto_scheme,
	CryptoSchemeFlag, GenericNumber,
};
use structopt::StructOpt;
use codec::{Codec, Encode, Decode};
use std::{str::FromStr, fmt::Debug};
use sp_runtime::{MultiSigner, MultiSignature, AccountId32};
use frame_utils::{SignedExtensionProvider, IndexFor, CallFor, AccountIdFor, AddressFor};
use crate::utils::create_extrinsic_for;
use sp_core::hexdisplay::HexDisplay;

type Bytes = Vec<u8>;

/// The `sign-transaction` command
#[derive(Debug, StructOpt)]
#[structopt(
	name = "sign-transaction",
	about = "Sign transaction from encoded Call.\
	Returns a signed and encoded UncheckedMortalCompactExtrinsic as hex."
)]
pub struct SignTransactionCmd {
	/// The secret key URI.
	#[structopt(long)]
	suri: String,

	/// The nonce.
	#[structopt(long)]
	nonce: GenericNumber,

	/// genesis hash, for signed extensions.
	#[structopt(long, parse(try_from_str = decode_hex))]
	prior_block_hash: Bytes,

	/// The call, hex-encoded.
	#[structopt(long, parse(try_from_str = decode_hex))]
	call: Bytes,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub keystore_params: KeystoreParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub crypto_scheme: CryptoSchemeFlag,
}

impl SignTransactionCmd {
	/// Run the command
	pub fn run<P>(&self) -> Result<(), Error>
		where
			P: SignedExtensionProvider + pallet_indices::Trait,
			<IndexFor<P> as FromStr>::Err: Debug,
			AccountIdFor<P>: From<AccountId32>,
			AddressFor<P>: From<AccountIdFor<P>>,
			CallFor<P>: Codec,
	{
		let nonce = self.nonce.parse::<IndexFor<P>>()?;
		let hash = <P::Hash as Decode>::decode(&mut &self.prior_block_hash[..])?;
		let call = CallFor::<P>::decode(&mut &self.call[..])?;
		let password = self.keystore_params.read_password()?;

		with_crypto_scheme!(
			self.crypto_scheme.scheme,
			print_ext<P>(&self.suri, password.as_ref().map(String::as_str), call, nonce, hash)
		)
	}
}


impl CliConfiguration for SignTransactionCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		Some(&self.keystore_params)
	}
}


fn print_ext<Pair, P>(
	uri: &str,
	pass: Option<&str>,
	call: CallFor<P>,
	nonce: IndexFor<P>,
	hash: P::Hash
) -> Result<(), Error>
	where
		Pair: sp_core::Pair,
		Pair::Public: Into<MultiSigner>,
		Pair::Signature: Into<MultiSignature>,
		P: SignedExtensionProvider + pallet_indices::Trait,
		AccountIdFor<P>: From<AccountId32>,
		AddressFor<P>: From<AccountIdFor<P>>,
		CallFor<P>: Codec,
{
	let signer = pair_from_suri::<Pair>(uri, pass);
	let extrinsic = create_extrinsic_for::<Pair, P, P::Call>(call, nonce, signer, hash)?;
	println!("0x{}", HexDisplay::from(&extrinsic.encode()));
	Ok(())
}