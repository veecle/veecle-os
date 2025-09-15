use anyhow::Result;
use can_dbc::DBC;
use proc_macro2::TokenStream;
use quote::quote;

use crate::dbc_ext::DBCExt;

mod actors;
mod messages;

fn database_comment(dbc: &DBC) -> String {
    let version = dbc
        .find_raw_attribute_string("DatabaseVersion")
        .unwrap_or("unknown");
    let bus = dbc
        .find_raw_attribute_string("BusType")
        .unwrap_or("unknown");
    let protocol = dbc
        .find_raw_attribute_string("ProtocolType")
        .unwrap_or("unknown");
    let compiler = dbc
        .find_raw_attribute_string("DatabaseCompiler")
        .unwrap_or("unknown");

    format!(" {protocol} v{version} for {bus} by {compiler}")
}

/// Generates a module for everything defined by the `dbc`.
///
/// `krate` should be a path to the `veecle-os-data-support-can` crate.
pub(crate) fn generate(options: &crate::Options, dbc: &DBC) -> Result<TokenStream> {
    let docs = database_comment(dbc);
    let messages = messages::generate(options, dbc)?;
    let actors = actors::generate(options, dbc)?;

    Ok(quote! {
        #![doc = #docs]

        #![allow(dead_code)]

        #messages
        #actors
    })
}
