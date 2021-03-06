// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Derive macro implementation of `PassBy` with the associated type set to `Inner` and of the
//! helper trait `PassByInner`.
//!
//! It is required that the type is a newtype struct, otherwise an error is generated.

use crate::utils::{generate_crate_access, generate_runtime_interface_include};

use syn::{DeriveInput, Result, Generics, parse_quote, Type, Data, Error, Fields, Ident};

use quote::quote;

use proc_macro2::{TokenStream, Span};

/// The derive implementation for `PassBy` with `Inner` and `PassByInner`.
pub fn derive_impl(mut input: DeriveInput) -> Result<TokenStream> {
	add_trait_bounds(&mut input.generics);
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
	let crate_include = generate_runtime_interface_include();
	let crate_ = generate_crate_access();
	let ident = input.ident;
	let (inner_ty, inner_name) = extract_inner_ty_and_name(&input.data)?;

	let access_inner = match inner_name {
		Some(ref name) => quote!(self.#name),
		None => quote!(self.0),
	};

	let from_inner = match inner_name {
		Some(name) => quote!(Self { #name: inner }),
		None => quote!(Self(inner)),
	};

	let res = quote! {
		const _: () = {
			#crate_include

			impl #impl_generics #crate_::pass_by::PassBy for #ident #ty_generics #where_clause {
				type PassBy = #crate_::pass_by::Inner<#ident, #inner_ty>;
			}

			impl #impl_generics #crate_::pass_by::PassByInner for #ident #ty_generics #where_clause {
				type Inner = #inner_ty;

				fn into_inner(self) -> Self::Inner {
					#access_inner
				}

				fn inner(&self) -> &Self::Inner {
					&#access_inner
				}

				fn from_inner(inner: Self::Inner) -> Self {
					#from_inner
				}
			}
		};
	};

	Ok(res)
}

/// Add the `RIType` trait bound to every type parameter.
fn add_trait_bounds(generics: &mut Generics) {
	let crate_ = generate_crate_access();

	generics.type_params_mut()
		.for_each(|type_param| type_param.bounds.push(parse_quote!(#crate_::RIType)));
}

/// Extract the inner type and optional name from given input data.
///
/// It also checks that the input data is a newtype struct.
fn extract_inner_ty_and_name(data: &Data) -> Result<(Type, Option<Ident>)> {
	if let Data::Struct(ref struct_data) = data {
		match struct_data.fields {
			Fields::Named(ref named) if named.named.len() == 1 => {
				let field = &named.named[0];
				return Ok((field.ty.clone(), field.ident.clone()))
			},
			Fields::Unnamed(ref unnamed) if unnamed.unnamed.len() == 1 => {
				let field = &unnamed.unnamed[0];
				return Ok((field.ty.clone(), field.ident.clone()))
			}
			_ => {},
		}
	}

	Err(
		Error::new(
			Span::call_site(),
			"Only newtype/one field structs are supported by `PassByInner`!",
		)
	)
}
