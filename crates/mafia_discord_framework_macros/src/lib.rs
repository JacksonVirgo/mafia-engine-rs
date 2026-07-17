use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Error, FnArg, Ident, ItemFn, Lit, Meta, Pat, PathArguments, Result, Token, Type,
    parse_macro_input, punctuated::Punctuated,
};

#[proc_macro_attribute]
pub fn slash_command(arguments: TokenStream, input: TokenStream) -> TokenStream {
    let arguments =
        parse_macro_input!(arguments with Punctuated::<Meta, Token![,]>::parse_terminated);
    match expand(
        arguments.into_iter().collect(),
        parse_macro_input!(input as ItemFn),
    ) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand(arguments: Vec<Meta>, mut function: ItemFn) -> Result<proc_macro2::TokenStream> {
    if function.sig.asyncness.is_none() {
        return Err(Error::new_spanned(
            function.sig.fn_token,
            "`#[slash_command]` requires an async function",
        ));
    }

    let command_description = description_from_meta(&arguments)?;
    let command_name = function.sig.ident.to_string();
    let public_name = function.sig.ident.clone();
    let handler_name = format_ident!("__{}_slash_command_handler", public_name);
    function.sig.ident = handler_name.clone();
    function.vis = syn::Visibility::Inherited;

    let mut inputs = function.sig.inputs.iter();
    let Some(FnArg::Typed(context)) = inputs.next() else {
        return Err(Error::new_spanned(
            &function.sig.inputs,
            "a slash command's first parameter must be `CommandContext`",
        ));
    };
    if !is_command_context(&context.ty) {
        return Err(Error::new_spanned(
            &context.ty,
            "a slash command's first parameter must be `CommandContext`",
        ));
    }

    let mut schemas = Vec::new();
    let mut extractions = Vec::new();
    let mut arguments = Vec::new();

    for input in inputs {
        let FnArg::Typed(parameter) = input else {
            return Err(Error::new_spanned(
                input,
                "methods cannot be slash commands",
            ));
        };
        let Pat::Ident(parameter_name) = parameter.pat.as_ref() else {
            return Err(Error::new_spanned(
                &parameter.pat,
                "slash command parameters must be named identifiers",
            ));
        };
        let name = parameter_name.ident.clone();
        let name_string = name.to_string();
        let description = description_from_attributes(&parameter.attrs)?
            .unwrap_or_else(|| format!("The {name_string} parameter"));
        let kind = parameter_kind(&parameter.ty)?;

        let schema = kind.schema(&name_string, &description);
        let extraction = kind.extract(&name, &name_string);
        schemas.push(schema);
        extractions.push(extraction);
        arguments.push(name);
    }

    let description = command_description.unwrap_or_else(|| format!("The {command_name} command"));
    for input in &mut function.sig.inputs {
        if let FnArg::Typed(parameter) = input {
            parameter
                .attrs
                .retain(|attribute| !attribute.path().is_ident("description"));
        }
    }

    Ok(quote! {
        #function

        pub fn #public_name() -> ::mafia_discord_framework::SlashCommand {
            ::mafia_discord_framework::SlashCommand::new(#command_name, #description)
                #(#schemas)*
                .handler(|context: ::mafia_discord_framework::CommandContext| async move {
                    #(#extractions)*
                    #handler_name(context, #(#arguments),*).await
                })
        }
    })
}

enum ParameterKind {
    String { optional: bool },
    Integer { optional: bool },
    Boolean { optional: bool },
}

impl ParameterKind {
    fn schema(&self, name: &str, description: &str) -> proc_macro2::TokenStream {
        match self {
            Self::String { optional: false } => quote!(.required_string(#name, #description)),
            Self::String { optional: true } => quote!(.optional_string(#name, #description)),
            Self::Integer { optional: false } => quote!(.required_integer(#name, #description)),
            Self::Integer { optional: true } => quote!(.optional_integer(#name, #description)),
            Self::Boolean { optional: false } => quote!(.required_boolean(#name, #description)),
            Self::Boolean { optional: true } => quote!(.optional_boolean(#name, #description)),
        }
    }

    fn extract(&self, variable: &Ident, name: &str) -> proc_macro2::TokenStream {
        match self {
            Self::String { optional: false } => {
                quote!(let #variable = context.required_string(#name)?.to_owned();)
            }
            Self::String { optional: true } => {
                quote!(let #variable = context.string(#name)?.map(str::to_owned);)
            }
            Self::Integer { optional: false } => {
                quote!(let #variable = context.required_integer(#name)?;)
            }
            Self::Integer { optional: true } => quote!(let #variable = context.integer(#name)?;),
            Self::Boolean { optional: false } => {
                quote!(let #variable = context.required_boolean(#name)?;)
            }
            Self::Boolean { optional: true } => quote!(let #variable = context.boolean(#name)?;),
        }
    }
}

fn parameter_kind(ty: &Type) -> Result<ParameterKind> {
    if let Some(name) = simple_type_name(ty) {
        return match name.as_str() {
            "String" => Ok(ParameterKind::String { optional: false }),
            "i64" => Ok(ParameterKind::Integer { optional: false }),
            "bool" => Ok(ParameterKind::Boolean { optional: false }),
            _ => Err(Error::new_spanned(
                ty,
                "unsupported slash command parameter; use String, i64, bool, or Option of one of them",
            )),
        };
    }

    let Type::Path(path) = ty else {
        return Err(Error::new_spanned(
            ty,
            "unsupported slash command parameter type",
        ));
    };
    let Some(segment) = path.path.segments.last() else {
        return Err(Error::new_spanned(
            ty,
            "unsupported slash command parameter type",
        ));
    };
    if segment.ident != "Option" {
        return Err(Error::new_spanned(
            ty,
            "unsupported slash command parameter; use String, i64, bool, or Option of one of them",
        ));
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(Error::new_spanned(ty, "Option requires one type parameter"));
    };
    let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
        return Err(Error::new_spanned(ty, "Option requires one type parameter"));
    };
    match simple_type_name(inner).as_deref() {
        Some("String") => Ok(ParameterKind::String { optional: true }),
        Some("i64") => Ok(ParameterKind::Integer { optional: true }),
        Some("bool") => Ok(ParameterKind::Boolean { optional: true }),
        _ => Err(Error::new_spanned(
            ty,
            "unsupported optional parameter; use Option<String>, Option<i64>, or Option<bool>",
        )),
    }
}

fn is_command_context(ty: &Type) -> bool {
    simple_type_name(ty).as_deref() == Some("CommandContext")
}

fn simple_type_name(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else {
        return None;
    };
    if !matches!(path.path.segments.last()?.arguments, PathArguments::None) {
        return None;
    }
    Some(path.path.segments.last()?.ident.to_string())
}

fn description_from_meta(metas: &[Meta]) -> Result<Option<String>> {
    let mut description = None;
    for meta in metas {
        let Meta::NameValue(name_value) = meta else {
            return Err(Error::new_spanned(meta, "use `description = \"...\"`"));
        };
        if !name_value.path.is_ident("description") {
            return Err(Error::new_spanned(
                meta,
                "unsupported slash command attribute",
            ));
        }
        description = Some(string_literal(&name_value.value)?);
    }
    Ok(description)
}

fn description_from_attributes(attributes: &[Attribute]) -> Result<Option<String>> {
    let mut description = None;
    for attribute in attributes {
        if !attribute.path().is_ident("description") {
            continue;
        }
        let Meta::NameValue(name_value) = &attribute.meta else {
            return Err(Error::new_spanned(
                attribute,
                "use `#[description = \"...\"]`",
            ));
        };
        description = Some(string_literal(&name_value.value)?);
    }
    Ok(description)
}

fn string_literal(expression: &syn::Expr) -> Result<String> {
    let syn::Expr::Lit(expression) = expression else {
        return Err(Error::new_spanned(
            expression,
            "description must be a string literal",
        ));
    };
    let Lit::Str(value) = &expression.lit else {
        return Err(Error::new_spanned(
            expression,
            "description must be a string literal",
        ));
    };
    Ok(value.value())
}
