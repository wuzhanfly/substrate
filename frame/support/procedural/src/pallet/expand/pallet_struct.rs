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

use crate::pallet::Def;

/// * Add derive trait on Pallet
/// * Implement GetPalletVersion on Pallet
/// * Implement OnGenesis on Pallet
/// * Implement ModuleErrorMetadata on Pallet
pub fn expand_pallet_struct(def: &mut Def) -> proc_macro2::TokenStream {
	let frame_support = &def.frame_support;
	let frame_system = &def.frame_system;
	let type_impl_gen = &def.type_impl_generics();
	let type_use_gen = &def.type_use_generics();
	let pallet_ident = &def.pallet_struct.pallet;
	let config_where_clause = &def.config.where_clause;

	let pallet_item = {
		let pallet_module_items = &mut def.item.content.as_mut().expect("Checked by def").1;
		let item = &mut pallet_module_items[def.pallet_struct.index];
		if let syn::Item::Struct(item) = item {
			item
		} else {
			unreachable!("Checked by pallet struct parser")
		}
	};

	pallet_item.attrs.push(syn::parse_quote!(
		#[derive(
			#frame_support::CloneNoBound,
			#frame_support::EqNoBound,
			#frame_support::PartialEqNoBound,
			#frame_support::RuntimeDebugNoBound,
		)]
	));

	let module_error_metadata = if let Some(error_def) = &def.error {
		let error_ident = &error_def.error;
		quote::quote!(
			impl<#type_impl_gen> #frame_support::error::ModuleErrorMetadata
				for #pallet_ident<#type_use_gen>
				#config_where_clause
			{
				fn metadata() -> &'static [#frame_support::error::ErrorMetadata] {
					<
						#error_ident<#type_use_gen> as #frame_support::error::ModuleErrorMetadata
					>::metadata()
				}
			}
		)
	} else {
		quote::quote!(
			impl<#type_impl_gen> #frame_support::error::ModuleErrorMetadata
				for #pallet_ident<#type_use_gen>
				#config_where_clause
			{
				fn metadata() -> &'static [#frame_support::error::ErrorMetadata] {
					&[]
				}
			}
		)
	};

	quote::quote!(
		#module_error_metadata

		// Implement `GetPalletVersion` for `Pallet`
		impl<#type_impl_gen> #frame_support::traits::GetPalletVersion
			for #pallet_ident<#type_use_gen>
			#config_where_clause
		{
			fn current_version() -> #frame_support::traits::PalletVersion {
				#frame_support::crate_to_pallet_version!()
			}

			fn storage_version() -> Option<#frame_support::traits::PalletVersion> {
				let key = #frame_support::traits::PalletVersion::storage_key::<
						<T as #frame_system::Config>::PalletInfo, Self
					>().expect("Every active pallet has a name in the runtime; qed");

				#frame_support::storage::unhashed::get(&key)
			}
		}

		// Implement `OnGenesis` for `Pallet`
		impl<#type_impl_gen> #frame_support::traits::OnGenesis
			for #pallet_ident<#type_use_gen>
			#config_where_clause
		{
			fn on_genesis() {
				#frame_support::crate_to_pallet_version!()
					.put_into_storage::<<T as #frame_system::Config>::PalletInfo, Self>();
			}
		}
	)
}
