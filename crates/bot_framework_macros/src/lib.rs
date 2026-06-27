use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, ExprLit, Fields, GenericArgument, Ident, ItemStruct, Lit, LitStr, Meta, Path,
    PathArguments, Token, Type, parse_macro_input,
};

// ---------- shared attribute parsing ----------

struct CommandArgs {
    name: Option<LitStr>,
    description: Option<LitStr>,
    subcommands: Vec<Path>,
    subcommand_groups: Vec<Path>,
    state: Option<Path>,
}

impl Parse for CommandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = CommandArgs {
            name: None,
            description: None,
            subcommands: Vec::new(),
            subcommand_groups: Vec::new(),
            state: None,
        };
        if input.is_empty() {
            return Ok(args);
        }
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            match meta {
                Meta::NameValue(nv) => {
                    let ident = nv
                        .path
                        .get_ident()
                        .ok_or_else(|| syn::Error::new_spanned(&nv.path, "expected identifier"))?
                        .to_string();
                    match ident.as_str() {
                        "name" => args.name = Some(expect_lit_str(&nv.value)?),
                        "description" => args.description = Some(expect_lit_str(&nv.value)?),
                        "state" => args.state = Some(expect_path(&nv.value)?),
                        other => {
                            return Err(syn::Error::new_spanned(
                                &nv.path,
                                format!("unknown argument `{other}`"),
                            ));
                        }
                    }
                }
                Meta::List(list) if list.path.is_ident("subcommands") => {
                    let paths = list.parse_args_with(
                        Punctuated::<Path, Token![,]>::parse_terminated,
                    )?;
                    args.subcommands.extend(paths);
                }
                Meta::List(list) if list.path.is_ident("subcommand_groups") => {
                    let paths = list.parse_args_with(
                        Punctuated::<Path, Token![,]>::parse_terminated,
                    )?;
                    args.subcommand_groups.extend(paths);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "expected `name = \"...\"`, `description = \"...\"`, `state = path`, `subcommands(...)`, or `subcommand_groups(...)`",
                    ));
                }
            }
        }
        Ok(args)
    }
}

fn expect_lit_str(expr: &Expr) -> syn::Result<LitStr> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        Ok(s.clone())
    } else {
        Err(syn::Error::new_spanned(expr, "expected string literal"))
    }
}

fn expect_path(expr: &Expr) -> syn::Result<Path> {
    if let Expr::Path(p) = expr {
        Ok(p.path.clone())
    } else {
        Err(syn::Error::new_spanned(expr, "expected a type path"))
    }
}

fn doc_comment(attrs: &[Attribute]) -> Option<String> {
    let mut out = String::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &attr.meta
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
        {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(s.value().trim());
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

fn default_state() -> Type {
    syn::parse_quote!(crate::State)
}

fn args_state_ty(args: &CommandArgs) -> Type {
    args.state
        .clone()
        .map(|p| Type::Path(syn::TypePath { qself: None, path: p }))
        .unwrap_or_else(default_state)
}

fn require_name_and_description(
    args: &CommandArgs,
    input: &ItemStruct,
    attr_label: &str,
) -> syn::Result<(String, String)> {
    let name = args.name.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            format!("`#[{attr_label}]` requires `name = \"...\"`"),
        )
    })?;
    let description = args.description.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            format!("`#[{attr_label}]` requires `description = \"...\"`"),
        )
    })?;
    Ok((name.value(), description.value()))
}

// ---------- #[command] / #[subcommand] on struct ----------

fn build_leaf_struct(
    attr: TokenStream,
    item: TokenStream,
    is_subcommand: bool,
) -> TokenStream {
    let args = parse_macro_input!(attr as CommandArgs);
    let input = parse_macro_input!(item as ItemStruct);

    let attr_label = if is_subcommand { "subcommand" } else { "command" };
    let (name_str, description) = match require_name_and_description(&args, &input, attr_label) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let struct_ident = input.ident.clone();
    let state_ty = args_state_ty(&args);

    let from_options = match generate_from_options(&struct_ident, &input.fields) {
        Ok(ts) => ts,
        Err(e) => return e.to_compile_error().into(),
    };

    let (trait_path, descriptor_ty, descriptor_body): (TokenStream2, TokenStream2, TokenStream2) = if is_subcommand {
        (
            quote!(::bot_framework::commands::SubcommandModule<#state_ty>),
            quote!(::bot_framework::commands::SubcommandDescriptor<#state_ty>),
            quote! {
                ::bot_framework::commands::SubcommandDescriptor {
                    name: #name_str,
                    description: #description,
                    options: <Self as ::bot_framework::commands::FromInteractionOptions>::option_specs,
                    handler: ::std::boxed::Box::new(|ctx, raw_opts| {
                        ::std::boxed::Box::pin(async move {
                            let instance = <Self as ::bot_framework::commands::FromInteractionOptions>::from_options(&raw_opts)?;
                            Self::run(instance, ctx).await
                        })
                    }),
                }
            },
        )
    } else {
        (
            quote!(::bot_framework::commands::CommandModule<#state_ty>),
            quote!(::bot_framework::commands::CommandDescriptor<#state_ty>),
            quote! {
                ::bot_framework::commands::CommandDescriptor {
                    name: #name_str,
                    description: #description,
                    kind: ::bot_framework::commands::CommandKind::Leaf(
                        ::bot_framework::commands::LeafHandler {
                            options: <Self as ::bot_framework::commands::FromInteractionOptions>::option_specs,
                            handler: ::std::boxed::Box::new(|ctx, raw_opts| {
                                ::std::boxed::Box::pin(async move {
                                    let instance = <Self as ::bot_framework::commands::FromInteractionOptions>::from_options(&raw_opts)?;
                                    Self::run(instance, ctx).await
                                })
                            }),
                        }
                    ),
                }
            },
        )
    };

    let expanded = quote! {
        #input

        #from_options

        impl #trait_path for #struct_ident {
            fn descriptor() -> #descriptor_ty {
                #descriptor_body
            }
        }
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    build_leaf_struct(attr, item, false)
}

#[proc_macro_attribute]
pub fn subcommand(attr: TokenStream, item: TokenStream) -> TokenStream {
    build_leaf_struct(attr, item, true)
}

// ---------- #[command_group] / #[subcommand_group] on struct ----------

fn build_group(
    attr: TokenStream,
    item: TokenStream,
    is_subcommand_group: bool,
) -> TokenStream {
    let args = parse_macro_input!(attr as CommandArgs);
    let input = parse_macro_input!(item as ItemStruct);

    let attr_label = if is_subcommand_group {
        "subcommand_group"
    } else {
        "command_group"
    };
    let (name_str, description) = match require_name_and_description(&args, &input, attr_label) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let struct_ident = input.ident.clone();
    let state_ty = args_state_ty(&args);
    let sub_paths = &args.subcommands;
    let group_paths = &args.subcommand_groups;

    if is_subcommand_group && !group_paths.is_empty() {
        return syn::Error::new_spanned(
            &input,
            "subcommand_groups is not allowed inside a #[subcommand_group] (Discord caps nesting at 2 levels)",
        )
        .to_compile_error()
        .into();
    }

    let (trait_path, descriptor_ty, body): (TokenStream2, TokenStream2, TokenStream2) = if is_subcommand_group {
        (
            quote!(::bot_framework::commands::SubcommandGroupModule<#state_ty>),
            quote!(::bot_framework::commands::SubcommandGroupDescriptor<#state_ty>),
            quote! {
                ::bot_framework::commands::SubcommandGroupDescriptor {
                    name: #name_str,
                    description: #description,
                    subcommands: vec![
                        #( <#sub_paths as ::bot_framework::commands::SubcommandModule<#state_ty>>::descriptor() ),*
                    ],
                }
            },
        )
    } else {
        (
            quote!(::bot_framework::commands::CommandModule<#state_ty>),
            quote!(::bot_framework::commands::CommandDescriptor<#state_ty>),
            quote! {
                ::bot_framework::commands::CommandDescriptor {
                    name: #name_str,
                    description: #description,
                    kind: ::bot_framework::commands::CommandKind::Group({
                        let mut entries: ::std::vec::Vec<::bot_framework::commands::SubcommandEntry<#state_ty>> = ::std::vec::Vec::new();
                        #(
                            entries.push(::bot_framework::commands::IntoSubcommandEntry::into_entry(
                                <#sub_paths as ::bot_framework::commands::SubcommandModule<#state_ty>>::descriptor()
                            ));
                        )*
                        #(
                            entries.push(::bot_framework::commands::IntoSubcommandEntry::into_entry(
                                <#group_paths as ::bot_framework::commands::SubcommandGroupModule<#state_ty>>::descriptor()
                            ));
                        )*
                        entries
                    }),
                }
            },
        )
    };

    let expanded = quote! {
        #input

        impl #trait_path for #struct_ident {
            fn descriptor() -> #descriptor_ty {
                #body
            }
        }
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn command_group(attr: TokenStream, item: TokenStream) -> TokenStream {
    build_group(attr, item, false)
}

#[proc_macro_attribute]
pub fn subcommand_group(attr: TokenStream, item: TokenStream) -> TokenStream {
    build_group(attr, item, true)
}

// ---------- FromInteractionOptions impl generation ----------

fn generate_from_options(struct_ident: &Ident, fields: &Fields) -> syn::Result<TokenStream2> {
    let (specs, ctor): (Vec<TokenStream2>, TokenStream2) = match fields {
        Fields::Unit => (Vec::new(), quote!(Self)),
        Fields::Named(named) => {
            let mut specs = Vec::new();
            let mut inits = Vec::new();
            for field in &named.named {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = field_ident.to_string().replace('_', "-");
                let description =
                    doc_comment(&field.attrs).unwrap_or_else(|| field_name.clone());
                let (extractor, spec_fn, is_optional) = classify_field_type(&field.ty);
                let required = !is_optional;
                specs.push(quote! {
                    ::bot_framework::commands::options::#spec_fn(#field_name, #description, #required)
                });
                inits.push(quote! {
                    #field_ident: ::bot_framework::commands::options::#extractor(options, #field_name)?
                });
            }
            (specs, quote!(Self { #( #inits ),* }))
        }
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                fields,
                "tuple structs are not supported as commands",
            ));
        }
    };

    Ok(quote! {
        impl ::bot_framework::commands::FromInteractionOptions for #struct_ident {
            fn from_options(
                options: &[::bot_framework::commands::__reexport::CommandDataOption],
            ) -> ::std::result::Result<Self, ::bot_framework::error::BotError> {
                Ok(#ctor)
            }
            fn option_specs() -> ::std::vec::Vec<::bot_framework::commands::__reexport::CommandOption> {
                vec![ #( #specs ),* ]
            }
        }
    })
}

fn classify_field_type(ty: &Type) -> (Ident, Ident, bool) {
    if let Some(inner) = option_inner(ty) {
        let (_, spec_fn) = primitive_kind(&inner);
        let extractor = format_ident!("optional_{}", primitive_extractor(&inner));
        (extractor, spec_fn, true)
    } else {
        let (extractor_kind, spec_fn) = primitive_kind(ty);
        let extractor = format_ident!("required_{}", extractor_kind);
        (extractor, spec_fn, false)
    }
}

fn primitive_kind(ty: &Type) -> (Ident, Ident) {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        match seg.ident.to_string().as_str() {
            "String" => return (format_ident!("string"), format_ident!("string_spec")),
            "i64" => return (format_ident!("integer"), format_ident!("integer_spec")),
            "bool" => return (format_ident!("bool"), format_ident!("bool_spec")),
            _ => {}
        }
    }
    (format_ident!("string"), format_ident!("string_spec"))
}

fn primitive_extractor(ty: &Type) -> &'static str {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        match seg.ident.to_string().as_str() {
            "String" => return "string",
            "i64" => return "integer",
            "bool" => return "bool",
            _ => {}
        }
    }
    "string"
}

fn option_inner(ty: &Type) -> Option<Type> {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && seg.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner.clone());
    }
    None
}
