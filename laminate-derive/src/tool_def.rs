use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Lit, Result};

/// Parse tool-level attributes from `#[tool(...)]`.
struct ToolAttrs {
    name: Option<String>,
    description: Option<String>,
    rename: Option<String>,
}

impl ToolAttrs {
    fn new() -> Self {
        Self {
            name: None,
            description: None,
            rename: None,
        }
    }

    fn parse(attrs: &[syn::Attribute]) -> Self {
        let mut result = Self::new();
        for attr in attrs {
            if attr.path().is_ident("tool") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.name = Some(s.value());
                        }
                    } else if meta.path.is_ident("description") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.description = Some(s.value());
                        }
                    } else if meta.path.is_ident("rename") {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.rename = Some(s.value());
                        }
                    }
                    Ok(())
                });
            }
        }
        result
    }
}

/// Extract doc comment text from attributes.
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(s) = &expr_lit.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .collect();

    if docs.is_empty() {
        None
    } else {
        Some(docs.join(" "))
    }
}

/// Map a Rust type to a JSON Schema type string.
fn json_schema_type(ty: &syn::Type) -> (String, bool) {
    // Returns (json_type, is_optional)
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let ident = seg.ident.to_string();
            match ident.as_str() {
                "String" => return ("string".to_string(), false),
                "bool" => return ("boolean".to_string(), false),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
                    return ("integer".to_string(), false);
                }
                "f32" | "f64" => return ("number".to_string(), false),
                "Option" => {
                    // Option<T> → T's type, marked optional
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                            let (inner_type, _) = json_schema_type(inner);
                            return (inner_type, true);
                        }
                    }
                    return ("string".to_string(), true);
                }
                "Vec" => return ("array".to_string(), false),
                _ => return ("object".to_string(), false),
            }
        }
    }
    ("string".to_string(), false)
}

/// Convert a PascalCase struct name to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn expand_tool_definition(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Get struct-level attributes
    let tool_attrs = ToolAttrs::parse(&input.attrs);

    // Tool name: explicit #[tool(name = "x")] or snake_case of struct name
    let tool_name = tool_attrs
        .name
        .unwrap_or_else(|| to_snake_case(&name.to_string()));

    // Tool description: explicit #[tool(description = "x")] or struct doc comment
    let tool_description = tool_attrs
        .description
        .or_else(|| extract_doc_comment(&input.attrs))
        .unwrap_or_else(|| format!("{} tool", tool_name));

    // Parse fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    &input,
                    "ToolDefinition only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input,
                "ToolDefinition can only be derived for structs",
            ));
        }
    };

    // Build property entries and required list
    let mut property_inserts = Vec::new();
    let mut required_names = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_attrs = ToolAttrs::parse(&field.attrs);

        // Parameter name: explicit rename or field name
        let param_name = field_attrs
            .rename
            .unwrap_or_else(|| field_ident.to_string());

        // Parameter description from doc comment
        let param_desc = extract_doc_comment(&field.attrs).unwrap_or_default();

        // JSON schema type
        let (json_type, is_optional) = json_schema_type(&field.ty);

        // Build the property object
        let prop_value = if param_desc.is_empty() {
            quote! {
                ::serde_json::json!({ "type": #json_type })
            }
        } else {
            quote! {
                ::serde_json::json!({ "type": #json_type, "description": #param_desc })
            }
        };

        property_inserts.push(quote! {
            properties.insert(#param_name.to_string(), #prop_value);
        });

        if !is_optional {
            required_names.push(param_name);
        }
    }

    let required_array: Vec<TokenStream> = required_names
        .iter()
        .map(|n| quote! { ::serde_json::json!(#n) })
        .collect();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Generate the tool definition JSON schema for this type.
            ///
            /// Returns a `serde_json::Value` matching the format expected by
            /// Anthropic and OpenAI tool/function calling APIs.
            pub fn tool_definition() -> ::serde_json::Value {
                let mut properties = ::serde_json::Map::new();
                #(#property_inserts)*

                ::serde_json::json!({
                    "name": #tool_name,
                    "description": #tool_description,
                    "input_schema": {
                        "type": "object",
                        "properties": ::serde_json::Value::Object(properties),
                        "required": [#(#required_array),*]
                    }
                })
            }
        }
    };

    Ok(expanded)
}
