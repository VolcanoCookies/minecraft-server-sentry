use std::f32::consts::E;

use darling::{FromDeriveInput, FromField};
use quote::ToTokens;
use syn::{DataEnum, DataStruct};

#[derive(Debug, Default)]
pub enum StructRepr {
    #[default]
    Flat,
    Json,
}

#[derive(Debug)]
pub enum PacketOpts {
    Struct {
        repr: StructRepr,
        data: DataStruct,
    },
    Enum {
        opts: EnumPacketOpts,
        data: DataEnum,
    },
}

#[derive(Debug)]
pub enum EnumPacketOpts {
    Repr(syn::Type),
    Index(syn::Type),
}

#[derive(Debug, Eq, PartialEq, FromDeriveInput)]
#[darling(attributes(packet))]
pub struct PacketStructAttr {
    repr: Option<String>,
}

#[derive(Debug, Eq, PartialEq, FromDeriveInput)]
#[darling(attributes(packet))]
pub struct PacketEnumAttr {
    repr: Option<String>,
    index: Option<String>,
}

pub fn spanned_error<T>(span: &syn::DeriveInput, msg: &str) -> Result<T, proc_macro2::TokenStream> {
    Err(syn::Error::new_spanned(span, msg).into_compile_error())
}

pub fn parse_packet_attrs(ast: &syn::DeriveInput) -> Result<PacketOpts, proc_macro2::TokenStream> {
    match &ast.data {
        syn::Data::Struct(data) => {
            let packet_attr = PacketStructAttr::from_derive_input(&ast)
                .expect("Failed to parse packet attribute");

            let repr = match packet_attr.repr {
                Some(repr) => match repr.as_str() {
                    "json" => StructRepr::Json,
                    _ => {
                        return spanned_error(ast, "Invalid repr attribute");
                    }
                },
                None => StructRepr::Flat,
            };

            let attrs = PacketOpts::Struct {
                repr,
                data: data.clone(),
            };

            Ok(attrs)
        }
        syn::Data::Enum(data) => {
            // If its a enum with int values, parse as repr and match into enum.
            // If its a enum with struct values, parse as index and match into struct specific enum.

            let packet_attr =
                PacketEnumAttr::from_derive_input(&ast).expect("Failed to parse packet attribute");

            if packet_attr.index.is_some() && packet_attr.repr.is_some() {
                return spanned_error(ast, "Cannot use both index and repr attributes on enum");
            }

            let opts = if let Some(type_str) = packet_attr.repr {
                let ty = syn::parse_str(&type_str).expect("Failed to parse repr type");
                EnumPacketOpts::Repr(ty)
            } else if let Some(type_str) = packet_attr.index {
                let ty = syn::parse_str(&type_str).expect("Failed to parse index type");
                EnumPacketOpts::Index(ty)
            } else {
                return spanned_error(ast, "Missing repr attribute");
            };

            let attrs = PacketOpts::Enum {
                opts: opts,
                data: data.clone(),
            };

            Ok(attrs)
        }
        _ => {
            return spanned_error(ast, "Unsupported data type");
        }
    }
}

#[derive(Debug, Eq, PartialEq, FromField)]
#[darling(attributes(packet))]
pub struct PacketFieldAttr {
    pub repr: Option<String>,
    pub len: Option<String>,
    pub cond: Option<String>,
}

pub enum FieldVecLen {
    Prefixed,
    Rest,
}

pub enum FieldCondition {
    Prefixed,
    Condition(syn::Expr),
}

pub struct FieldOpts {
    pub repr: Option<syn::Type>,
    pub len: Option<FieldVecLen>,
    pub cond: Option<FieldCondition>,
}

pub fn parse_field_attrs(field: &syn::Field) -> Result<FieldOpts, proc_macro2::TokenStream> {
    let field_attr = PacketFieldAttr::from_field(field)
        .map_err(|e| syn::Error::new_spanned(field, e).into_compile_error())?;

    let repr = field_attr
        .repr
        .as_ref()
        .map(|s| syn::parse_str(s).expect("Failed to parse repr attribute"));

    let len = if field.ty.to_token_stream().to_string() == "Vec < u8 >" {
        if let Some(len) = field_attr.len {
            match len.as_str() {
                "prefixed" => Some(FieldVecLen::Prefixed),
                "rest" => Some(FieldVecLen::Rest),
                _ => {
                    return Err(syn::Error::new_spanned(field, "Invalid len attribute")
                        .into_compile_error());
                }
            }
        } else {
            None
        }
    } else {
        if field_attr.len.is_some() {
            return Err(
                syn::Error::new_spanned(field, "len attribute is only valid for Vec<u8>")
                    .into_compile_error(),
            );
        }
        None
    };

    let cond = if let syn::Type::Path(type_path) = &field.ty {
        if is_path_ident(&type_path.path, "Option") {
            if field_attr.cond.is_some() {
                let cond_expr = syn::parse_str(&field_attr.cond.as_ref().unwrap())
                    .expect("Failed to parse condition expression");
                Some(FieldCondition::Condition(cond_expr))
            } else {
                Some(FieldCondition::Prefixed)
            }
        } else if field_attr.cond.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "cond attribute is only valid for Option<T> types",
            )
            .into_compile_error());
        } else {
            None
        }
    } else if field_attr.cond.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "cond attribute is only valid for Option<T> types",
        )
        .into_compile_error());
    } else {
        None
    };

    Ok(FieldOpts { repr, len, cond })
}

fn is_path_ident(path: &syn::Path, ident: &str) -> bool {
    path.segments.len() == 1 && path.segments[0].ident == ident
}
