mod attrs;

use std::fmt::Display;

use attrs::{parse_field_attrs, FieldVecLen, PacketOpts};
use convert_case::Casing;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, spanned::Spanned, Field, Fields};

use crate::attrs::{EnumPacketOpts, FieldCondition};

extern crate proc_macro;

#[proc_macro_derive(Packet, attributes(packet_id, packet_state, packet_direction, packet))]
pub fn packet_derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).expect("Failed to parse input");

    let packet = impl_packet_derive(&ast).unwrap_or_else(|err| err);
    let reader = impl_packet_readable_derive(&ast).unwrap_or_else(|err| err);
    let writer = impl_packet_writable_derive(&ast).unwrap_or_else(|err| err);

    quote! {
        #packet
        #reader
        #writer
    }
    .into()
}

#[proc_macro_derive(PacketReadable, attributes(enum_type, packet))]
pub fn packet_readable_derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).expect("Failed to parse input");

    impl_packet_readable_derive(&ast)
        .unwrap_or_else(|err| err)
        .into()
}

#[proc_macro_derive(PacketWritable, attributes(packet))]
pub fn packet_writable_derive(input: TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).expect("Failed to parse input");

    impl_packet_writable_derive(&ast)
        .unwrap_or_else(|err| err)
        .into()
}

fn spanned_error<T: ToTokens, D: Display>(
    tokens: T,
    message: D,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    Err(syn::Error::new_spanned(tokens, message)
        .to_compile_error()
        .into())
}

// (ident, reader, writer)
fn field_parser(
    field: &Field,
    prefix: Option<&str>,
) -> Result<
    (
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ),
    proc_macro2::TokenStream,
> {
    fn spanned_error<T: ToTokens, D: Display>(
        tokens: T,
        message: D,
    ) -> Result<
        (
            proc_macro2::TokenStream,
            proc_macro2::TokenStream,
            proc_macro2::TokenStream,
        ),
        proc_macro2::TokenStream,
    > {
        Err(syn::Error::new_spanned(tokens, message)
            .to_compile_error()
            .into())
    }

    let ident = field.ident.as_ref().unwrap();
    let prefix: proc_macro2::TokenStream = match prefix {
        Some(p) => p.parse().unwrap(),
        None => quote! {},
    };
    let ty = &field.ty;

    let opts = parse_field_attrs(field)?;

    let (ident, mut reader, writer) = match ty {
        syn::Type::Path(ty) => {
            let ident = ident.to_token_stream();
            if let Some(repr) = opts.repr {
                let ident_raw = syn::Ident::new(&format!("{}_raw", ident), ident.span());
                let reader = quote! {
                    let #ident_raw: #repr = packet::read::PacketReadable::read(&mut raw)?;
                    let #ident = #ident_raw.into();
                };
                let writer = quote! {
                    let #ident_raw: #repr = (*#prefix #ident).into();
                    packet::write::PacketWritable::write(&#ident_raw, &mut raw)?;
                };

                (ident, reader, writer)
            } else if let Some(FieldVecLen::Rest) = opts.len {
                let ident_greedy = syn::Ident::new(&format!("{}_greedy", ident), ident.span());
                let inner_ty = match ty
                    .path
                    .segments
                    .first()
                    .expect("Segment missing")
                    .ident
                    .to_string()
                    .as_str()
                {
                    "Vec" => {
                        let inner = &ty
                            .path
                            .segments
                            .first()
                            .expect("Segment argument")
                            .arguments;
                        match inner {
                            syn::PathArguments::AngleBracketed(args) => {
                                let inner = args.args.first().expect("Expected inner type");
                                match inner {
                                    syn::GenericArgument::Type(ty) => ty,
                                    _ => {
                                        return spanned_error(
                                            inner,
                                            "expected type argument for Vec",
                                        );
                                    }
                                }
                            }
                            _ => {
                                return spanned_error(inner, "expected angle bracketed arguments");
                            }
                        }
                    }
                    _ => {
                        return spanned_error(ty, "expected Vec type");
                    }
                };

                let reader = quote! {
                    let #ident_greedy: packet::types::GreedyVec<#inner_ty> = packet::read::PacketReadable::read(&mut raw)?;
                    let #ident = #ident_greedy.0;
                };
                // Prefix is reference, so we dereference it immediately
                let writer = quote! {
                    let #ident_greedy = packet::types::GreedyVec((*#prefix #ident).clone());
                    packet::write::PacketWritable::write(&#ident_greedy, &mut raw)?;
                };

                (ident, reader, writer)
            } else {
                let reader = quote! {
                    let #ident: #ty = packet::read::PacketReadable::read(&mut raw)?;
                };
                let writer = quote! {
                    packet::write::PacketWritable::write(#prefix #ident, &mut raw)?;
                };

                (ident, reader, writer)
            }
        }
        _ => return spanned_error(ty, format!("unsupported field type {:?}", ty)),
    };

    if let Some(cond) = opts.cond {
        match cond {
            FieldCondition::Prefixed => {
                reader = quote! {
                    let present: bool = packet::read::PacketReadable::read(&mut raw)?;
                    let #ident = if present {
                        #reader
                        #ident
                    } else {
                        None
                    };
                };
            }
            FieldCondition::Condition(expr) => {
                reader = quote! {
                    let #ident = if #expr {
                        #reader
                        #ident
                    } else {
                        None
                    };
                };
            }
        }
    }

    Ok((ident, reader, writer))
}

type FieldParserResult = Result<
    (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ),
    proc_macro2::TokenStream,
>;

fn extract_fields(ast: &syn::DeriveInput) -> FieldParserResult {
    fn spanned_error<T: ToTokens, D: Display>(tokens: T, message: D) -> FieldParserResult {
        Err(syn::Error::new_spanned(tokens, message)
            .to_compile_error()
            .into())
    }

    if let syn::Data::Struct(data) = &ast.data {
        return named_fields_parser(&data.fields, Some("&self.")).map_err(|err| err.into());
    } else {
        spanned_error(ast, "expected struct")
    }
}

fn named_fields_parser(
    fields: &Fields,
    prefix: Option<&str>,
) -> Result<
    (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ),
    proc_macro2::TokenStream,
> {
    fn spanned_error<T: ToTokens, D: Display>(tokens: T, message: D) -> FieldParserResult {
        Err(syn::Error::new_spanned(tokens, message)
            .to_compile_error()
            .into())
    }

    if let syn::Fields::Named(fields) = &fields {
        let fields = fields
            .named
            .iter()
            .map(|f| field_parser(f, prefix))
            .collect::<Vec<_>>();
        let mut idents = Vec::new();
        let mut readers = Vec::new();
        let mut writers = Vec::new();

        for field in fields {
            let (ident, reader, writer) = field?;
            idents.push(ident);
            readers.push(reader);
            writers.push(writer);
        }

        Ok((idents, readers, writers))
    } else {
        spanned_error(fields, "expected named fields")
    }
}

fn impl_packet_derive(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    // Get the name of the type for which this macro is applied.
    let name = &ast.ident;

    let packet_id = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("packet_id") {
            attr.parse_args::<syn::LitInt>().ok()
        } else {
            None
        }
    });
    let packet_id = match packet_id {
        Some(id) => id,
        _ => {
            return spanned_error(ast, "packet_id attribute is required");
        }
    };

    let packet_state = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("packet_state") {
            attr.parse_args::<syn::Variant>().ok()
        } else {
            None
        }
    });
    let packet_state = match packet_state {
        Some(state) => state.ident,
        _ => {
            return spanned_error(ast, "packet_state attribute is required");
        }
    };

    let packet_direction = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("packet_direction") {
            attr.parse_args::<syn::Variant>().ok()
        } else {
            None
        }
    });
    let packet_direction = match packet_direction {
        Some(direction) => direction.ident,
        _ => {
            return spanned_error(ast, "packet_direction attribute is required");
        }
    };

    let upper_snake = name.to_string().to_case(convert_case::Case::Constant);
    let descriptor_ident = syn::Ident::new(&format!("_{}_DESCRIPTOR", upper_snake), name.span());

    let descriptor = quote! {
        packet::registry::PacketDescriptor {
            id: #packet_id,
            state: packet::ConnectionState::#packet_state,
            name: stringify!(#name),
            direction: packet::PacketDirection::#packet_direction,
            read_fn: |mut reader| {
                let res = <#name as packet::read::PacketReadable>::read(&mut reader);
                match res {
                    Ok(packet) => Ok(Box::new(packet)),
                    Err(e) => Err(e),
                }
            },
            write_fn: |packet, mut writer| {
                let coerced = packet.into_any();
                let packet = match coerced.downcast::<#name>() {
                    Ok(packet) => *packet,
                    Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to downcast")),
                };
                <#name as packet::write::PacketWritable>::write(&packet, &mut writer)?;
                Ok(())
            },
        };
    };

    let gen = quote! {
        #[linkme::distributed_slice(packet::registry::PACKET_REGISTRY)]
        static #descriptor_ident: packet::registry::PacketDescriptor = #descriptor

        impl packet::PacketData for #name {
            fn packet_id(&self) -> i32 {
                <Self as packet::Packet>::PACKET_ID
            }

            fn packet_state(&self) -> packet::ConnectionState {
                <Self as packet::Packet>::PACKET_STATE
            }

            fn packet_direction(&self) -> packet::PacketDirection {
                <Self as packet::Packet>::PACKET_DIRECTION
            }

            fn descriptor(&self) -> &'static packet::registry::PacketDescriptor {
                & #descriptor_ident
            }

            fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
        }

        impl packet::Packet for #name {

            const PACKET_ID: i32 = #packet_id;
            const PACKET_STATE: packet::ConnectionState = packet::ConnectionState::#packet_state;
            const PACKET_DIRECTION: packet::PacketDirection = packet::PacketDirection::#packet_direction;
            const PACKET_DESCRIPTOR: packet::registry::PacketDescriptor = #descriptor

            // Read and validate packet id, then read as regular struct
            fn read_packet<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let packet_id: packet::types::VarInt = packet::read::PacketReadable::read(reader)?;

                if Self::PACKET_ID != packet_id.0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid packet id",
                    ));
                }

                // Read self as regular struct
                let parsed = packet::read::PacketReadable::read(reader)?;

                Ok(parsed)
            }

            // Write packet id then write as regular struct
            fn write_packet<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                let packet_id = packet::types::VarInt(Self::PACKET_ID);
                packet::write::PacketWritable::write(&packet_id, writer)?;

                packet::write::PacketWritable::write(self, writer)?;

                Ok(())
            }
        }
    };

    Ok(gen.into())
}

fn impl_packet_readable_derive(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    // Get the name of the type for which this macro is applied.
    let name = &ast.ident;

    let opts = attrs::parse_packet_attrs(ast)?;

    match opts {
        PacketOpts::Struct { repr, .. } => {
            let (idents, readers, _) = extract_fields(ast)?;

            match repr {
                attrs::StructRepr::Flat => {
                    let gen = quote! {
                        impl packet::read::PacketReadable for #name {
                            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                                let mut raw = reader;

                                #(
                                    #readers
                                )*

                                Ok(Self {
                                    #(
                                        #idents,
                                    )*
                                })
                            }
                        }
                    };

                    return Ok(gen.into());
                }
                attrs::StructRepr::Json => {
                    let gen = quote! {
                        impl packet::read::PacketReadable for #name {
                            fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                                let string: String = packet::read::PacketReadable::read(reader)?;
                                let parsed = serde_json::from_str(&string)?;
                                Ok(parsed)
                            }
                        }
                    };

                    return Ok(gen.into());
                }
            }
        }
        PacketOpts::Enum { opts, data } => {
            let parse = match opts {
                EnumPacketOpts::Repr(repr) => {
                    let mut variants = Vec::new();
                    data.variants.iter().for_each(|v| {
                        let ident = &v.ident;
                        let value = match &v.discriminant {
                            Some((_, expr)) => expr,
                            None => panic!("Expected enum variant to have a value"),
                        };

                        variants.push(quote! {
                            #value => Self::#ident,
                        });
                    });

                    quote! {
                        let value: #repr = packet::read::PacketReadable::read(reader)?;

                        let variant = match value.into() {
                            #(
                                #variants
                            )*
                            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid enum value")),
                        };
                    }
                }
                EnumPacketOpts::Index(repr) => {
                    let mut variants = Vec::new();
                    let mut idx: i32 = 0;
                    for v in data.variants.iter() {
                        let ident = &v.ident;
                        let (idents, reader, _) = named_fields_parser(&v.fields, Some("&self."))?;

                        let reader_impl = quote! {
                            #idx => {
                                let mut raw = reader;
                                #(
                                    #reader
                                )*

                                Self::#ident {
                                    #(
                                        #idents,
                                    )*
                                }
                            }
                        };
                        idx += 1;

                        variants.push(reader_impl);
                    }

                    quote! {
                        let index: #repr = packet::read::PacketReadable::read(reader)?;
                        let index: i32 = index.into();

                        let variant = match index {
                            #(
                                #variants
                            )*
                            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid enum index")),
                        };
                    }
                }
            };

            let gen = quote! {
                impl packet::read::PacketReadable for #name {
                    fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                        #parse

                        Ok(variant)
                    }
                }
            };

            return Ok(gen.into());
        }
    }
}

fn impl_packet_writable_derive(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    // Get the name of the type for which this macro is applied.
    let name = &ast.ident;

    let opts = attrs::parse_packet_attrs(ast)?;

    match &opts {
        PacketOpts::Struct { .. } => {}
        PacketOpts::Enum { .. } => {}
    }

    match &opts {
        PacketOpts::Struct { repr, .. } => {
            let (_, _, writers) = extract_fields(ast)?;

            match repr {
                attrs::StructRepr::Flat => {
                    let gen = quote! {
                        impl packet::write::PacketWritable for #name {
                            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                                let mut raw = writer;

                                #(
                                    #writers
                                )*

                                Ok(())
                            }
                        }
                    };

                    return Ok(gen.into());
                }
                attrs::StructRepr::Json => {
                    let gen = quote! {
                        impl packet::write::PacketWritable for #name {
                            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                                let string = serde_json::to_string(self)?;
                                packet::write::PacketWritable::write(&string, writer)?;
                                Ok(())
                            }
                        }
                    };

                    return Ok(gen.into());
                }
            }
        }
        PacketOpts::Enum { opts, data } => {
            match opts {
                EnumPacketOpts::Repr(repr) => {
                    let mut variants = Vec::new();
                    data.variants.iter().for_each(|v| {
                        let ident = &v.ident;
                        let value = match &v.discriminant {
                            Some((_, expr)) => expr,
                            None => panic!("Expected enum variant to have a value"),
                        };

                        variants.push(quote! {
                            #name::#ident => #repr::from(#value),
                        });
                    });

                    return Ok(quote! {
                        impl Into<#repr> for &#name {
                            fn into(self) -> #repr {
                                match self {
                                    #(
                                        #variants
                                    )*
                                }
                            }
                        }

                        impl packet::write::PacketWritable for #name {
                            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                                let value: #repr = self.into();
                                packet::write::PacketWritable::write(&value, writer)?;
                                Ok(())
                            }
                        }
                    }.into());
                }
                EnumPacketOpts::Index(repr) => {
                    let mut variants = Vec::new();
                    let mut variants_index = Vec::new();
                    let mut idx: i32 = 0;
                    for v in data.variants.iter() {
                        let ident = &v.ident;
                        let (idents, _, writer) = named_fields_parser(&v.fields, None)?;

                        let variant_index = quote! {
                            #name::#ident { .. } => #idx,
                        };
                        variants_index.push(variant_index);

                        let writer_impl = quote! {
                            #name::#ident { #(#idents),* } => {
                                let mut raw = writer;
                                #(
                                    #writer
                                )*
                            }
                        };
                        idx += 1;

                        variants.push(writer_impl);
                    }

                    return Ok(quote! {
                        impl packet::write::PacketWritable for #name {
                            fn index(&self) -> i32 {
                                match self {
                                    #(
                                        #variants_index
                                    )*
                                }
                            }

                            fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                                let idx: #repr = self.index().into();
                                packet::write::PacketWritable::write(&idx, writer)?;
                                match self {
                                    #(
                                        #variants
                                    )*
                                }
                                Ok(())
                            }
                        }
                    }.into());
                }
            };
        }
    }
}
