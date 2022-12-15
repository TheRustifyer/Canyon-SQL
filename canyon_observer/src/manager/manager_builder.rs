use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Generics, Visibility};

use super::entity::CanyonEntity;

/// Builds the TokenStream that contains the user defined struct
pub fn generate_user_struct(canyon_entity: &CanyonEntity) -> TokenStream {
    let fields = &canyon_entity.get_attrs_as_token_stream();

    let struct_name: &Ident = &canyon_entity.struct_name;
    let struct_visibility: &Visibility = &canyon_entity.vis;
    let struct_generics: &Generics = &canyon_entity.generics;
    let struct_attrs: &Vec<Attribute> = &canyon_entity.attrs;

    quote! {
        #(#struct_attrs)*
        #struct_visibility struct #struct_name #struct_generics {
            #(#fields),*
        }
    }
}

/// Auto-generated enum to represent every field of the related type
/// as a variant of an enum that it's named with the concatenation
/// of the type identifier + Field
///
/// The idea it's to have a representation of the field name as an enum
/// variant, avoiding to let the user passing around Strings and instead,
/// passing variants of a concrete enumeration type, that when required,
/// will be called though macro code to obtain the &str representation
/// of the field name.
pub fn generate_enum_with_fields(canyon_entity: &CanyonEntity) -> TokenStream {
    let ty = &canyon_entity.struct_name;
    let struct_name = canyon_entity.struct_name.to_string();
    let enum_name = Ident::new((struct_name + "Field").as_str(), Span::call_site());

    let fields_names = &canyon_entity.get_fields_as_enum_variants();
    let match_arms_str = &canyon_entity.create_match_arm_for_get_variant_as_str(&enum_name);
    let match_arms = &canyon_entity.create_match_arm_for_get_variant_as_string(&enum_name);

    let visibility = &canyon_entity.vis;
    let generics = &canyon_entity.generics;

    quote! {
        #[derive(Clone, Debug)]
        #[allow(non_camel_case_types)]
        #[allow(unused_variables)]
        #[allow(dead_code)]
        /// Auto-generated enum to represent every field of the related type
        /// as a variant of an enum that it's named with the concatenation
        /// of the type identifier + Field
        ///
        /// The idea it's to have a representation of the field name as an enum
        /// variant, avoiding the user to have to pass around Strings and instead,
        /// passing variants of a concrete enumeration type, that when required,
        /// will be called though macro code to obtain the &str representation
        /// of the field name.
        ///
        /// That's particulary useful in Canyon when working with queries being constructed
        /// through the [`QueryBuilder`], when one of the methods requieres to get
        /// a column name (which is the name of some field of the type) as a parameter
        ///
        /// ```
        /// pub struct League {
        ///     id: i32,
        ///     name: String
        /// }
        ///
        /// #[derive(Debug)]
        /// #[allow(non_camel_case_types)]
        /// pub enum LeagueField {
        ///     id(i32),
        ///     name(String)
        /// }
        /// ```
        #visibility enum #enum_name #generics {
            #(#fields_names),*
        }

        impl #generics canyon_sql::crud::bounds::FieldIdentifier<#ty> for #generics #enum_name #generics {
            fn as_str(&self) -> &'static str {
                match *self {
                    #(#match_arms_str),*
                }
            }
            fn field_name_as_str(self) -> String {
                match self {
                    #(#match_arms),*
                }
            }
        }
    }
}

/// Autogenerated Rust Enum type that contains as many variants
/// with inner value as fields has the structure to which it relates
///
/// The type of the inner value `(Enum::Variant(SomeType))` is the same
/// that the field that the variant represents
pub fn generate_enum_with_fields_values(canyon_entity: &CanyonEntity) -> TokenStream {
    let ty = &canyon_entity.struct_name;
    let struct_name = canyon_entity.struct_name.to_string();
    let enum_name = Ident::new((struct_name + "FieldValue").as_str(), Span::call_site());

    let fields_names = &canyon_entity.get_fields_as_enum_variants_with_value();
    let match_arms = &canyon_entity.create_match_arm_for_relate_fields_with_values(&enum_name);

    let visibility = &canyon_entity.vis;

    quote! {
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        #[allow(unused_variables)]
        #[allow(dead_code)]
        /// Auto-generated enumeration to represent each field of the related
        /// type as a variant, which can support and contain a value of the field data type.
        ///
        /// ```
        /// pub struct League {
        ///     id: i32,
        ///     name: String,
        ///     opt: Option<String>
        /// }
        ///
        /// #[derive(Debug)]
        /// #[allow(non_camel_case_types)]
        /// pub enum LeagueFieldValue {
        ///     id(i32),
        ///     name(String)
        ///     opt(Option<String>)
        /// }
        /// ```
        #visibility enum #enum_name<'a> {
            #(#fields_names),*
        }

        impl<'a> canyon_sql::crud::bounds::FieldValueIdentifier<'a, #ty> for #enum_name<'a> {
            fn value(self) -> (&'static str, &'a dyn QueryParameters<'a>) {
                match self {
                    #(#match_arms),*
                }
            }
        }
    }
}
