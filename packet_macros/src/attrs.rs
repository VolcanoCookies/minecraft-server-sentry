use darling::{FromDeriveInput, FromField};
use syn::{DataEnum, DataStruct};

#[derive(Debug, Default)]
pub enum StructRepr {
    #[default]
    Flat,
    Json,
}

#[derive(Debug)]
pub enum PacketOpts {
    Struct { repr: StructRepr, data: DataStruct },
    Enum { repr: syn::Type, data: DataEnum },
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
            // If its a enum with int values, generate the reader.

            let packet_attr =
                PacketEnumAttr::from_derive_input(&ast).expect("Failed to parse packet attribute");

            let repr = if let Some(type_str) = packet_attr.repr {
                let ty = syn::parse_str(&type_str).expect("Failed to parse repr type");
                ty
            } else {
                return spanned_error(ast, "Missing repr attribute");
            };

            let attrs = PacketOpts::Enum {
                repr: repr,
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
}

pub struct FieldOpts {
    pub repr: Option<syn::Type>,
}

pub fn parse_field_attrs(field: &syn::Field) -> Result<FieldOpts, proc_macro2::TokenStream> {
    let field_attr = PacketFieldAttr::from_field(field)
        .map_err(|e| syn::Error::new_spanned(field, e).into_compile_error())?;

    let repr = field_attr.repr.as_ref().map(|s| syn::parse_str(s).unwrap());

    Ok(FieldOpts { repr })
}
