#![feature(attr_literals)]
#![recursion_limit="128"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;
extern crate spoilers;


use std::collections::HashMap;
use proc_macro::TokenStream;
use quote::ToTokens;


/// Util to parse a derive input from a derive decored token.
fn parse_derive_input(input: TokenStream) -> syn::DeriveInput {
    let s = input.to_string();
    syn::parse_derive_input(&s).unwrap()
}


/// Util to parse derive attributes from a derive input.
fn parse_derive_attibutes<'a>(ast: syn::DeriveInput) -> HashMap<String, syn::MetaItem> {
    ast.attrs.iter().map(|x| (x.name().to_owned(), x.value.clone())).collect()
}

/// State composed of macro variables used as an util to generate
/// macro implementations.
struct MetaResourceConfig {
    pub ast: syn::DeriveInput,
}


impl MetaResourceConfig {
    pub fn new(input: syn::DeriveInput) -> MetaResourceConfig {
        MetaResourceConfig {ast: input}
    }

    pub fn endpoint(&self) -> String {
        let attr_items = parse_derive_attibutes(self.ast.clone());
        attr_items.get("endpoint").map_or(
            self.struct_name().as_ref().to_owned(),
            |attr| match attr {
                &syn::MetaItem::NameValue(_, ref literal) => {
                    match literal {
                        &syn::Lit::Str(ref path, _) => path.to_owned(),
                        _ => panic!("Endpoint must be a string")
                    }
                },
                _ => panic!("Endpoint must be a string")
            }
        )
    }

    pub fn table_name(&self) -> syn::Ident {
        let attr_items = parse_derive_attibutes(self.ast.clone());
        syn::Ident::new(
            attr_items.get("table_name").map_or(
                self.struct_name().as_ref(),
                |attr| match attr {
                    &syn::MetaItem::NameValue(_, ref literal) => {
                        match literal {
                            &syn::Lit::Str(ref path, _) => path,
                            _ => panic!("Endpoint must be a string")
                        }
                    },
                    _ => panic!("Endpoint must be a string")
                }
            )
        )
    }

    pub fn struct_name(&self) -> syn::Ident {
        self.ast.ident.clone()
    }

    pub fn model_name(&self) -> syn::Ident {
        syn::Ident::new(format!("{}Model", self.struct_name()))
    }

    pub fn form_name(&self) -> syn::Ident {
        syn::Ident::new(format!("{}Form", self.struct_name()))
    }

    pub fn filter_name(&self) -> syn::Ident {
        syn::Ident::new(format!("{}Filter", self.struct_name()))
    }

    pub fn method_name(&self, verb: &str) -> syn::Ident {
        let ref_name = self.struct_name().as_ref().to_lowercase();
        syn::Ident::new(format!("{}_{}", ref_name, verb))
    }

    pub fn fields(&self) -> &[syn::Field] {
        match self.ast.body {
            syn::Body::Struct(ref data) => data.fields(),
            _ => panic!("Resources must be structs.")
        }
    }
}


#[proc_macro_derive(Resource, attributes(endpoint))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    impl_resource(&parse_derive_input(input)).parse().unwrap()
}


#[proc_macro_derive(CollectionGet, attributes(endpoint))]
pub fn derive_collection_get(input: TokenStream) -> TokenStream {
    impl_collection_get(&parse_derive_input(input)).parse().unwrap()
}


#[proc_macro_derive(CollectionCreate, attributes(endpoint))]
pub fn derive_collection_create(input: TokenStream) -> TokenStream {
    impl_collection_create(&parse_derive_input(input)).parse().unwrap()
}


fn impl_resource(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let struct_name = config.struct_name();
    let model_name = config.model_name();
    let form_name = config.form_name();
    let filter_name = config.filter_name();
    let table_name = config.table_name().as_ref().to_owned();
    let endpoint = config.endpoint().to_owned();

    let model_fields: Vec<quote::Tokens> = config.fields().iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        quote!{
            pub #ident: #ty,
        }
    }).collect();
    let form_fields: Vec<quote::Tokens> = config.fields().iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        quote!{
            pub #ident: #ty,
        }
    }).collect();

    println!("{:?}", form_fields);
    quote! {
        #[derive(Queryable, Serialize, Deserialize)]
        pub struct #model_name {
            pub id: i32,
            #(#model_fields)*
        }

        #[derive(Insertable, Serialize, Deserialize)]
        #[table_name=#table_name]
        pub struct #form_name {
            #(#form_fields)*
        }

        #[derive(Serialize, Deserialize)]
        pub struct #filter_name {
        }

        impl Resource for #struct_name {
            const ENDPOINT: &'static str = #endpoint;
        }
    }
}


fn impl_collection_get(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let endpoint = config.endpoint().to_owned();
    let method_name = config.method_name("get");
    let struct_name = config.struct_name();
    let filter_name = config.filter_name();
    let model_name = config.model_name();
    let table_name = config.table_name();
    quote! {
        #[get(#endpoint, format = "application/json")]
        fn #method_name(context: Context) -> JsonValue {
            let data = #struct_name::list(#filter_name {}, &*context);
            JsonValue(json!({"data": data.expect("error")}))
        }
    }
}


fn impl_collection_create(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let endpoint = config.endpoint().to_owned();
    let method_name = config.method_name("create");
    let struct_name = config.struct_name();
    let form_name = config.form_name();
    let model_name = config.model_name();
    let table_name = config.table_name();

    quote! {
        #[post(#endpoint, format = "application/json", data = "<message>")]
        fn #method_name(message: Json<Value>, context: Context) -> JsonValue {
            let new: #form_name = serde_json::from_value(message.0).unwrap();
            let created = #struct_name::create(new, &*context);
            JsonValue(json!({"data": created.expect("error")}))
        }
    }
}


#[proc_macro_derive(PgStorageBackend, attributes(endpoint))]
pub fn derive_pg_storage_backend(input: TokenStream) -> TokenStream {
    impl_pg_storage_backend(&parse_derive_input(input)).parse().unwrap()
}


fn impl_pg_storage_backend(ast: &syn::DeriveInput) -> quote::Tokens {
    let config = MetaResourceConfig::new(ast.clone());
    let struct_name = config.struct_name();
    let form_name = config.form_name();
    let model_name = config.model_name();
    let filter_name = config.filter_name();
    let table_name = config.table_name();

    quote! {

        impl spoilers::storage::StorageBackend<#form_name,#model_name,#filter_name,PgConnection> for #struct_name {

            fn create<'a>(form: #form_name, conn: &'a PgConnection) ->
                    Result<#model_name, StorageBackendError> {

                let created: #model_name = diesel::insert(&form).into(#table_name::table)
                    .get_result(conn)
                    .expect("Error saving new post");
                Ok(created)
            }

            fn list<'a>(filters: #filter_name, conn: &'a PgConnection) ->
                    Result<Vec<#model_name>, StorageBackendError> {

                use #table_name::dsl::*;
                let results = #table_name.limit(10)
                    .load::<#model_name>(&*conn)
                    .expect("Error loading events");
                Ok(results)
            }
        }
    }
}
