extern crate proc_macro;

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Fields, Visibility, parse_macro_input, ItemFn, Type
};


mod canyon_macro;

use canyon_macro::{_user_body_builder, _wire_data_on_canyon_register};
use canyon_observer::CANYON_REGISTER;


/// Macro for handling the entry point to the program. 
/// 
/// Avoids the user to write the tokio attribute and
/// the async modifier to the main fn
/// TODO Check for the _meta attribute metadata when necessary
#[proc_macro_attribute]
pub fn canyon(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    // get the function this attribute is attached to
    let func = parse_macro_input!(input as ItemFn);
    let sign = func.sig;
    let body = func.block;

    // The code written by the Canyon Manager
    let mut canyon_manager_tokens: Vec<TokenStream> = Vec::new();
    // Builds the code that Canyon needs in it's initialization
    _wire_data_on_canyon_register(&mut canyon_manager_tokens);

    // The code written by the user
    let mut macro_tokens: Vec<TokenStream> = Vec::new();
    // Builds the code that represents the user written code
    _user_body_builder(body, &mut macro_tokens);
    

    let tok = quote! {
        use canyon_sql::tokio;
        #[tokio::main]
        async #sign {
            {     
                #(#canyon_manager_tokens)*
            }

            #(#macro_tokens)*
        }
    };
    
    tok.into()
}


/// Takes data from the struct annotated with macro to fill the Canyon Register
/// where lives the data that Canyon needs to work in `managed mode`
#[proc_macro_attribute]
pub fn canyon_managed(_meta: CompilerTokenStream, input: CompilerTokenStream) -> CompilerTokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let fields = fields_with_types(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => panic!("Field names can only be derived for structs"),
        }
    );

    // Notifies the observer that an observable must be registered on the system
    // In other words, adds the data of the structure to the Canyon Register
    unsafe { CANYON_REGISTER.push(ty.to_string()); }
    println!("Observable <{}> added to the register", ty.to_string());

    
    let struct_fields = fields.iter().map(|(_vis, ident, ty)| {
        quote! {  
            #vis #ident: #ty
        }
    });

    let (_impl_generics, ty_generics, _where_clause) = 
        generics.split_for_impl();

    let tokens = quote! {
        pub struct #ty <#ty_generics> {
            #(#struct_fields),*
        }
    };

    tokens.into()
}


/// Allows the implementors to auto-derive de `crud-operations` trait, which defines the methods
/// that will perform the database communication and that will query against the db.
#[proc_macro_derive(CanyonCRUD)]
pub fn crud_operations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_crud_operations_trait_for_struct(&ast)
}


fn impl_crud_operations_trait_for_struct(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let ty = &ast.ident;
    let tokens = quote! {
        #[async_trait]
        impl canyon_sql::crud::CrudOperations<#ty> for #ty { }
        impl canyon_sql::crud::Transaction<#ty> for #ty { }
    };
    tokens.into()
}


#[proc_macro_derive(CanyonMapper)]
pub fn implement_row_mapper_for_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Gets the data from the AST
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);
    
    // Retrieves the table name automatically from the Struct identifier
    // or from the TODO: #table_name = 'user_defined_db_table_name' 
    let table_name: String = database_table_name_from_struct(ty);

    // Recoves the identifiers of the struct's members
    let fields = filter_fields(
        match ast.data {
            syn::Data::Struct(ref s) => &s.fields,
            _ => panic!("Field names can only be derived for structs"),
        }
    );

    // Creates the TokenStream for wire the column names into the 
    // Canyon RowMapper
    let field_names_for_row_mapper = fields.iter().map(|(_vis, ident)| {
        let ident_name = ident.to_string();
        quote! {  
            #ident: row.try_get(#ident_name)
                .expect(format!("Failed to retrieve the {} field", #ident_name).as_ref())
        }
    });

    // Get the generics identifiers
    let (impl_generics, ty_generics, where_clause) = 
        generics.split_for_impl();


    let tokens = quote! {
        impl #impl_generics #ty #ty_generics
            #where_clause
        {
            // Find all  // Select registers by columns not enabled yet
            #vis async fn find_all() -> Vec<#ty> {
                <#ty as CrudOperations<#ty>>::__find_all(#table_name, &[])
                    .await
                    .as_response::<#ty>()
            }

            // Find by ID
            #vis async fn find_by_id(id: i32) -> #ty {
                <#ty as CrudOperations<#ty>>::__find_by_id(#table_name, id)
                    .await
                    .as_response::<#ty>()[0].clone()
            }

        }

        impl RowMapper<Self> for #ty {
            fn deserialize(row: &Row) -> Self {
                Self {
                    #(#field_names_for_row_mapper),*
                }
            }
        }
    };

    tokens.into()
}


fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .map(|field| 
            (field.vis.clone(), field.ident.as_ref().unwrap().clone()) 
        )
        .collect::<Vec<_>>()
}


fn fields_with_types(fields: &Fields) -> Vec<(Visibility, Ident, Type)> {
    fields
        .iter()
        .map(|field| 
            (field.vis.clone(), 
            field.ident.as_ref().unwrap().clone(),
            field.ty.clone()
        ) 
        )
        .collect::<Vec<_>>()
}


/// Parses a syn::Identifier to get a snake case database name from the type identifier
fn database_table_name_from_struct(ty: &Ident) -> String {

    let struct_name: String = String::from(ty.to_string());
    let mut table_name: String = String::new();
    
    let mut index = 0;
    for char in struct_name.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase()); 
                }
                _ => table_name.push(char)
            }
        }   
    }

    table_name
}