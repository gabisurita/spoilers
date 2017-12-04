use syn;
use quote;

use utils::parse_derive_attibutes;


/// State composed of macro variables used as an util to generate
/// macro implementations.
pub struct MetaResource {
    pub ast: syn::DeriveInput,
}


impl MetaResource {
    pub fn new(input: syn::DeriveInput) -> Self {
        Self {ast: input}
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

    pub fn impl_resource(&self) -> quote::Tokens {
        let struct_name = self.struct_name();
        let model_name = self.model_name();
        let form_name = self.form_name();
        let filter_name = self.filter_name();
        let table_name = self.table_name().as_ref().to_owned();

        let model_fields: Vec<quote::Tokens> = self.fields().iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote!{
                pub #ident: #ty,
            }
        }).collect();
        let form_fields: Vec<quote::Tokens> = self.fields().iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote!{
                pub #ident: #ty,
            }
        }).collect();

        let collection_get = self.impl_collection_get();
        let collection_create = self.impl_collection_create();

        quote! {
            #[derive(Queryable, Serialize, Deserialize, Debug)]
            pub struct #model_name {
                pub id: i32,
                #(#model_fields)*
            }

            #[derive(Insertable, Serialize, Deserialize, Debug)]
            #[table_name=#table_name]
            pub struct #form_name {
                #(#form_fields)*
            }

            #[derive(Serialize, Deserialize)]
            pub struct #filter_name {
            }

            impl Resource for #struct_name {
            }

            #collection_get

            #collection_create
        }
    }

    pub fn impl_collection_get(&self) -> quote::Tokens {
        let method_name = self.method_name("get");
        let filter_name = self.filter_name();

        quote! {
            #[get("/", format = "application/json")]
            fn #method_name(context: Context) -> rocket_contrib::JsonValue {
                let data = context.list(#filter_name {});
                rocket_contrib::JsonValue(json!({"data": data.expect("error")}))
            }
        }
    }


    pub fn impl_collection_create(&self) -> quote::Tokens {
        let method_name = self.method_name("create");
        let form_name = self.form_name();

        quote! {
            #[post("/", format = "application/json", data = "<message>")]
            fn #method_name(message: rocket_contrib::Json<serde_json::Value>, context: Context)
                    -> Result<rocket_contrib::JsonValue, rocket::response::Failure> {
                let new: #form_name = match serde_json::from_value(message.0) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(rocket::response::Failure(rocket::http::Status::BadRequest));
                    }
                };
                let created = context.create(new);
                Ok(rocket_contrib::JsonValue(json!({"data": created.expect("error")})))
            }
        }
    }


    pub fn impl_pg_storage_backend(&self) -> quote::Tokens {
        let form_name = self.form_name();
        let model_name = self.model_name();
        let filter_name = self.filter_name();
        let table_name = self.table_name();

        quote! {

            impl spoilers::storage::ResourceStorage<#form_name,#model_name,#filter_name>
                    for Context {

                fn create<'a>(&self, form: #form_name) ->
                        Result<#model_name, ResourceStorageError> {

                    let created: #model_name = diesel::insert(&form).into(#table_name::table)
                        .get_result(&*self.db)
                        .expect("Error saving new post");
                    Ok(created)
                }

                fn bulk_create<'a>(&self, form: Vec<#form_name>) ->
                        Result<(), ResourceStorageError> {

                    let created: #model_name = diesel::insert(&form).into(#table_name::table)
                        .get_result(&*self.db)
                        .expect("Error saving new post");
                    Ok(())
                }

                fn list<'a>(&self, _filters: #filter_name) ->
                        Result<Vec<#model_name>, ResourceStorageError> {

                    use #table_name::dsl::*;
                    let results = #table_name.limit(10)
                        .load::<#model_name>(&*self.db)
                        .expect("Error loading events");
                    Ok(results)
                }
            }
        }
    }

    pub fn impl_redshift_storage_backend(&self) -> quote::Tokens {
        let struct_name = self.struct_name();
        let form_name = self.form_name();
        let model_name = self.model_name();
        let filter_name = self.filter_name();
        let table_name = self.table_name();
        let queue_name = self.table_name().as_ref().to_owned();

        quote! {
            impl #struct_name {
                pub fn sync<'a>(pool: &'a ConnectionPool, period: Duration) {
                    use std::{thread, time};
                    use redis::Commands;
                    let db_conn = match pool.db_pool.get() {
                        Ok(conn) => conn,
                        Err(_) => {return;}
                    };
                    let queue_conn = match pool.queue_pool.get() {
                        Ok(conn) => conn,
                        Err(_) => {return;}
                    };

                    let context = Context {
                        db: db_conn,
                        queue: queue_conn,
                    };

                    let _th = thread::spawn(move || {
                        loop {
                            let cached: Vec<String> = context.queue.lrange(#queue_name, 0, -1)
                                                                   .unwrap_or_default();


                            let cache_results: Vec<#form_name> = cached.iter().map(|s| {
                                serde_json::from_str(s.as_ref()).unwrap()
                            }).collect();

                            if cache_results.len() > 0 {
                                context.bulk_create(cache_results).unwrap();
                                let _: i32 = context.queue.del(#queue_name).unwrap();
                            }
                            thread::sleep(period);
                        }
                    });
                }
            }

            impl spoilers::storage::ResourceStorage<#form_name,#model_name,#filter_name>
                    for Context {

                fn create<'a>(&self, form: #form_name) ->
                        Result<#model_name, ResourceStorageError> {
                    use redis::Commands;
                    let serialized: String = serde_json::to_string(&form).unwrap();
                    let _: i32 = self.queue.rpush(#queue_name, serialized.clone()).unwrap();

                    let mut model_json: serde_json::Value =
                        serde_json::from_str(serialized.as_ref()).unwrap();

                    model_json["id"] = json!(-1);
                    let result: #model_name = serde_json::from_value(model_json).unwrap();
                    Ok(result)
                }

                fn bulk_create<'a>(&self, form: Vec<#form_name>) ->
                        Result<(), ResourceStorageError> {
                    let ingest = RedshiftIngest::new(&self.db);
                    ingest.process(
			#queue_name.to_owned(),
			format!("{}/{}", "spoilers-development", #queue_name.to_owned()),
			 vec![]
		    );
                    Ok(())
                }

                fn list<'a>(&self, _filters: #filter_name) ->
                        Result<Vec<#model_name>, ResourceStorageError> {

                    use redis::Commands;
                    use #table_name::dsl::*;
                    let mut db_results = #table_name
                        .limit(1000)
                        .load::<#model_name>(&*self.db)
                        .expect("Error loading events");
                    let cached: Vec<String> = self.queue.lrange(#queue_name, 0, -1).unwrap();
                    let cache_results: Vec<#model_name> = cached.iter().map(|s| {
                        let mut model_json: serde_json::Value =
                            serde_json::from_str(s.as_ref()).unwrap();
                        model_json["id"] = json!(-1);
                        serde_json::from_value(model_json).unwrap()
                    }).collect();
                    db_results.extend(cache_results);
                    Ok(db_results)
                }
            }
        }
    }
}
