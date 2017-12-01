#![feature(attr_literals)]
#![recursion_limit="256"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;
extern crate spoilers;

mod resource;
mod storage;
mod utils;

use proc_macro::TokenStream;

use resource::*;
use storage::*;
use utils::*;


#[proc_macro_derive(Resource, attributes(endpoint))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    MetaResource::new(parse_derive_input(input)).impl_resource().parse().unwrap()
}


#[proc_macro_derive(PgResourceStorage, attributes(endpoint, table_name))]
pub fn derive_pg_storage_backend(input: TokenStream) -> TokenStream {
    MetaResource::new(parse_derive_input(input)).impl_pg_storage_backend()
                                                .parse().unwrap()
}


#[proc_macro_derive(RedshiftResourceStorage, attributes(endpoint, table_name))]
pub fn derive_redshift_storage_backend(input: TokenStream) -> TokenStream {
    MetaResource::new(parse_derive_input(input)).impl_redshift_storage_backend()
                                                .parse().unwrap()
}


#[proc_macro_derive(PostgreStorage)]
pub fn derive_postgre_storage(input: TokenStream) -> TokenStream {
    impl_postgre_storage(&parse_derive_input(input)).parse().unwrap()
}


#[proc_macro_derive(RedshiftStorage)]
pub fn derive_redshift_storage(input: TokenStream) -> TokenStream {
    impl_redshift_storage(&parse_derive_input(input)).parse().unwrap()
}
