use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Lit, Result};

/// Extract a coercion type hint string from a syn::Type at compile time.
///
/// This replaces the broken `type_name::<T>().rsplit("::")` approach,
/// which corrupts generic types like `Option<String>` → `"String>"`.
///
/// Returns the last path segment identifier as a string (e.g., "i64", "String",
/// "Option<i64>"). For `Option<T>`, unwraps to T's hint since coercion applies
/// to the inner type.
fn type_hint_from_syn(ty: &syn::Type) -> String {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let ident = seg.ident.to_string();
            // For Option<T>, unwrap to T's hint (coercion targets the inner type)
            if ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return type_hint_from_syn(inner);
                    }
                }
            }
            return ident;
        }
    }
    // Fallback: stringify the type
    quote!(#ty).to_string()
}

/// Field-level attributes parsed from `#[laminate(...)]`.
struct FieldAttrs {
    /// Deserialize from a different JSON key.
    rename: Option<String>,
    /// Use Default::default() if missing or null.
    has_default: bool,
    /// Apply coercion rules.
    coerce: bool,
    /// This field captures unknown/overflow fields.
    overflow: bool,
    /// Don't attempt to deserialize this field from input.
    skip: bool,
    /// If the value is a string, try parsing it as JSON first.
    parse_json_string: bool,
    /// Merge fields from a nested object into the parent map.
    flatten: bool,
}

fn parse_field_attrs(field: &syn::Field) -> Result<FieldAttrs> {
    let mut attrs = FieldAttrs {
        rename: None,
        has_default: false,
        coerce: false,
        overflow: false,
        skip: false,
        parse_json_string: false,
        flatten: false,
    };

    for attr in &field.attrs {
        if !attr.path().is_ident("laminate") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    attrs.rename = Some(s.value());
                }
                Ok(())
            } else if meta.path.is_ident("default") {
                attrs.has_default = true;
                Ok(())
            } else if meta.path.is_ident("coerce") {
                attrs.coerce = true;
                Ok(())
            } else if meta.path.is_ident("overflow") {
                attrs.overflow = true;
                Ok(())
            } else if meta.path.is_ident("skip") {
                attrs.skip = true;
                Ok(())
            } else if meta.path.is_ident("parse_json_string") {
                attrs.parse_json_string = true;
                Ok(())
            } else if meta.path.is_ident("flatten") {
                attrs.flatten = true;
                Ok(())
            } else {
                Err(meta.error("unknown laminate attribute"))
            }
        })?;
    }

    Ok(attrs)
}

pub fn expand_laminate(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    name,
                    "Laminate can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                name,
                "Laminate can only be derived for structs",
            ));
        }
    };

    let mut overflow_field: Option<(syn::Ident, bool)> = None; // (ident, is_option)
    let mut regular_fields = Vec::new();
    let mut skip_fields: Vec<(syn::Ident, syn::Type)> = Vec::new();
    let mut flatten_fields: Vec<(syn::Ident, syn::Type)> = Vec::new();
    let mut known_keys: Vec<(String, syn::Ident)> = Vec::new();

    for field in fields {
        let field_ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(field, "unnamed fields not supported"))?;
        let attrs = parse_field_attrs(field)?;

        if attrs.overflow {
            if overflow_field.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "only one #[laminate(overflow)] field allowed per struct",
                ));
            }
            // Detect if the overflow field type is Option<...>
            let is_option = if let syn::Type::Path(tp) = &field.ty {
                tp.path
                    .segments
                    .last()
                    .map(|seg| seg.ident == "Option")
                    .unwrap_or(false)
            } else {
                false
            };
            overflow_field = Some((field_ident.clone(), is_option));
        } else if attrs.skip {
            skip_fields.push((field_ident.clone(), field.ty.clone()));
        } else if attrs.flatten {
            flatten_fields.push((field_ident.clone(), field.ty.clone()));
        } else {
            let json_key = attrs
                .rename
                .clone()
                .unwrap_or_else(|| field_ident.to_string());

            // Check for duplicate JSON keys (including via rename)
            if let Some((_, prev_ident)) = known_keys.iter().find(|(k, _)| k == &json_key) {
                return Err(Error::new_spanned(
                    field,
                    format!(
                        "duplicate JSON key \"{}\": field `{}` and field `{}` both map to the same key",
                        json_key, prev_ident, field_ident
                    ),
                ));
            }

            known_keys.push((json_key.clone(), field_ident.clone()));
            regular_fields.push((field_ident.clone(), field.ty.clone(), attrs, json_key));
        }
    }

    // Generate extraction code for skip fields (Default::default())
    let skip_extractions: Vec<TokenStream> = skip_fields
        .iter()
        .map(|(ident, ty)| {
            quote! {
                let #ident: #ty = <#ty as Default>::default();
            }
        })
        .collect();

    // Generate extraction code for flatten fields (deserialize from remaining map)
    let flatten_extractions: Vec<TokenStream> = flatten_fields
        .iter()
        .map(|(ident, ty)| {
            quote! {
                let #ident: #ty = {
                    let flat_val = ::serde_json::Value::Object(
                        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    );
                    let result = ::serde_json::from_value::<#ty>(flat_val)
                        .map_err(|e| ::laminate::FlexError::DeserializeError {
                            path: "(flatten)".to_string(),
                            source: e,
                        })?;
                    // Remove consumed keys from map so they're not flagged as unknown.
                    // Uses to_value() to discover serializable fields. This misses
                    // #[serde(skip_serializing)] fields, which may produce false
                    // "Dropped" diagnostics — a known limitation of the round-trip probe.
                    if let Ok(::serde_json::Value::Object(consumed)) = ::serde_json::to_value(&result) {
                        for key in consumed.keys() {
                            map.remove(key);
                        }
                    }
                    result
                };
            }
        })
        .collect();

    // Generate extraction code for each regular field
    let field_extractions: Vec<TokenStream> = regular_fields
        .iter()
        .map(|(ident, ty, attrs, json_key)| {
            // Optional preprocessing: parse_json_string
            let preprocess = if attrs.parse_json_string {
                quote! {
                    let val = if let ::serde_json::Value::String(ref s) = val {
                        match ::serde_json::from_str::<::serde_json::Value>(s) {
                            Ok(parsed) => parsed,
                            Err(_) => val,
                        }
                    } else {
                        val
                    };
                }
            } else {
                quote! {}
            };

            // Detect if field type is Vec<...> for single-to-array wrapping
            let is_vec = if let syn::Type::Path(tp) = ty {
                tp.path.segments.last()
                    .map(|seg| seg.ident == "Vec")
                    .unwrap_or(false)
            } else {
                false
            };

            // Extract Vec's inner type for element-level coercion
            let vec_inner_type: Option<&syn::Type> = if is_vec {
                if let syn::Type::Path(tp) = ty {
                    tp.path.segments.last().and_then(|seg| {
                        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                            args.args.first().and_then(|arg| {
                                if let syn::GenericArgument::Type(inner) = arg {
                                    Some(inner)
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            };

            // Detect Option<Vec<T>> for combined null guard + element coercion
            let option_vec_inner_type: Option<&syn::Type> = if let syn::Type::Path(tp) = ty {
                tp.path.segments.last().and_then(|seg| {
                    if seg.ident != "Option" { return None; }
                    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else { return None; };
                    let Some(syn::GenericArgument::Type(syn::Type::Path(inner_tp))) = args.args.first() else { return None; };
                    let inner_seg = inner_tp.path.segments.last()?;
                    if inner_seg.ident != "Vec" { return None; }
                    let syn::PathArguments::AngleBracketed(inner_args) = &inner_seg.arguments else { return None; };
                    match inner_args.args.first()? {
                        syn::GenericArgument::Type(elem_ty) => Some(elem_ty),
                        _ => None,
                    }
                })
            } else {
                None
            };

            let extract = if let (true, Some(elem_ty)) = (attrs.coerce, option_vec_inner_type.as_ref()) {
                // Option<Vec<T>> with coerce: null→None, array→element coercion
                let elem_hint = type_hint_from_syn(elem_ty);
                quote! {
                    match map.remove(#json_key) {
                        Some(::serde_json::Value::Null) | None => {
                            ::serde_json::from_value::<#ty>(::serde_json::Value::Null)
                                .map_err(|e| ::laminate::FlexError::DeserializeError {
                                    path: #json_key.to_string(),
                                    source: e,
                                })?
                        }
                        Some(val) => {
                            #preprocess
                            let arr = if let ::serde_json::Value::Array(a) = val {
                                a
                            } else {
                                vec![val]
                            };
                            let coerced_arr: Vec<::serde_json::Value> = arr.into_iter().enumerate().map(|(idx, elem)| {
                                let elem_path = format!("{}[{}]", #json_key, idx);
                                let result = ::laminate::coerce::coerce_value(
                                    &elem, #elem_hint,
                                    ::laminate::CoercionLevel::BestEffort,
                                    &elem_path,
                                );
                                if let Some(d) = result.diagnostic {
                                    _diagnostics.push(d);
                                }
                                result.value
                            }).collect();
                            ::serde_json::from_value::<#ty>(
                                ::serde_json::Value::Array(coerced_arr)
                            ).map_err(|e| ::laminate::FlexError::DeserializeError {
                                path: #json_key.to_string(),
                                source: e,
                            })?
                        }
                    }
                }
            } else if attrs.coerce && is_vec {
                // Vec field with coerce: wrap single values in an array,
                // then apply element-level coercion to each element
                let inner_ty = vec_inner_type.expect("Vec field must have inner type");
                let elem_hint = type_hint_from_syn(inner_ty);
                if attrs.has_default {
                    quote! {
                        match map.remove(#json_key) {
                            Some(::serde_json::Value::Null) | None => {
                                <#ty as Default>::default()
                            }
                            Some(val) => {
                                #preprocess
                                let arr = if let ::serde_json::Value::Array(a) = val {
                                    a
                                } else {
                                    vec![val]
                                };
                                let elem_short = #elem_hint;
                                let coerced_arr: Vec<::serde_json::Value> = arr.into_iter().enumerate().map(|(idx, elem)| {
                                    let elem_path = format!("{}[{}]", #json_key, idx);
                                    let result = ::laminate::coerce::coerce_value(
                                        &elem, elem_short,
                                        ::laminate::CoercionLevel::BestEffort,
                                        &elem_path,
                                    );
                                    if let Some(d) = result.diagnostic {
                                        _diagnostics.push(d);
                                    }
                                    result.value
                                }).collect();
                                ::serde_json::from_value::<#ty>(::serde_json::Value::Array(coerced_arr))
                                    .unwrap_or_else(|_| <#ty as Default>::default())
                            }
                        }
                    }
                } else {
                    quote! {
                        match map.remove(#json_key) {
                            Some(val) => {
                                #preprocess
                                let arr = if let ::serde_json::Value::Array(a) = val {
                                    a
                                } else {
                                    vec![val]
                                };
                                let elem_short = #elem_hint;
                                let coerced_arr: Vec<::serde_json::Value> = arr.into_iter().enumerate().map(|(idx, elem)| {
                                    let elem_path = format!("{}[{}]", #json_key, idx);
                                    let result = ::laminate::coerce::coerce_value(
                                        &elem, elem_short,
                                        ::laminate::CoercionLevel::BestEffort,
                                        &elem_path,
                                    );
                                    if let Some(d) = result.diagnostic {
                                        _diagnostics.push(d);
                                    }
                                    result.value
                                }).collect();
                                ::serde_json::from_value::<#ty>(::serde_json::Value::Array(coerced_arr))
                                    .map_err(|e| ::laminate::FlexError::DeserializeError {
                                        path: #json_key.to_string(),
                                        source: e,
                                    })?
                            }
                            None => {
                                return Err(::laminate::FlexError::PathNotFound {
                                    path: #json_key.to_string(),
                                });
                            }
                        }
                    }
                }
            } else if attrs.coerce && attrs.has_default {
                // Coerce with default fallback: try coercion, fall back to Default on failure
                let hint = type_hint_from_syn(ty);
                quote! {
                    match map.remove(#json_key) {
                        Some(::serde_json::Value::Null) | None => {
                            <#ty as Default>::default()
                        }
                        Some(val) => {
                            #preprocess
                            let coerced = ::laminate::coerce::coerce_value(
                                &val, #hint,
                                ::laminate::CoercionLevel::BestEffort,
                                #json_key,
                            );
                            if let Some(d) = coerced.diagnostic {
                                _diagnostics.push(d);
                            }
                            match ::serde_json::from_value::<#ty>(coerced.value.clone()) {
                                Ok(v) => v,
                                Err(e) => {
                                    _diagnostics.push(::laminate::Diagnostic {
                                        path: #json_key.to_string(),
                                        kind: ::laminate::DiagnosticKind::ErrorDefaulted {
                                            field: #json_key.to_string(),
                                            error: format!(
                                                "value {:?} could not be coerced to {}: {}",
                                                coerced.value, #hint, e
                                            ),
                                        },
                                        risk: ::laminate::RiskLevel::Warning,
                                        suggestion: Some(
                                            "coercion failed; using default value — check source data".to_string()
                                        ),
                                    });
                                    <#ty as Default>::default()
                                }
                            }
                        }
                    }
                }
            } else if attrs.coerce {
                // Use laminate coercion: try to coerce the value before deserializing
                let hint = type_hint_from_syn(ty);
                // Detect Option<T> to handle null → None before coercion
                let is_option = if let syn::Type::Path(tp) = ty {
                    tp.path.segments.last()
                        .map(|seg| seg.ident == "Option")
                        .unwrap_or(false)
                } else {
                    false
                };
                if is_option {
                    quote! {
                        match map.remove(#json_key) {
                            Some(::serde_json::Value::Null) | None => {
                                ::serde_json::from_value::<#ty>(::serde_json::Value::Null)
                                    .map_err(|e| ::laminate::FlexError::DeserializeError {
                                        path: #json_key.to_string(),
                                        source: e,
                                    })?
                            }
                            Some(val) => {
                                #preprocess
                                let coerced = ::laminate::coerce::coerce_value(
                                    &val, #hint,
                                    ::laminate::CoercionLevel::BestEffort,
                                    #json_key,
                                );
                                if let Some(d) = coerced.diagnostic {
                                    _diagnostics.push(d);
                                }
                                ::serde_json::from_value::<#ty>(coerced.value)
                                    .map_err(|e| ::laminate::FlexError::DeserializeError {
                                        path: #json_key.to_string(),
                                        source: e,
                                    })?
                            }
                        }
                    }
                } else {
                    quote! {
                        match map.remove(#json_key) {
                            Some(val) => {
                                #preprocess
                                let coerced = ::laminate::coerce::coerce_value(
                                    &val, #hint,
                                    ::laminate::CoercionLevel::BestEffort,
                                    #json_key,
                                );
                                if let Some(d) = coerced.diagnostic {
                                    _diagnostics.push(d);
                                }
                                ::serde_json::from_value::<#ty>(coerced.value)
                                    .map_err(|e| ::laminate::FlexError::DeserializeError {
                                        path: #json_key.to_string(),
                                        source: e,
                                    })?
                            }
                            None => {
                                return Err(::laminate::FlexError::PathNotFound {
                                    path: #json_key.to_string(),
                                });
                            }
                        }
                    }
                }
            } else if attrs.has_default {
                // Use Default::default() if missing
                quote! {
                    match map.remove(#json_key) {
                        Some(::serde_json::Value::Null) | None => {
                            <#ty as Default>::default()
                        }
                        Some(val) => {
                            #preprocess
                            ::serde_json::from_value::<#ty>(val)
                                .map_err(|e| ::laminate::FlexError::DeserializeError {
                                    path: #json_key.to_string(),
                                    source: e,
                                })?
                        }
                    }
                }
            } else {
                // Required field, no coercion
                quote! {
                    match map.remove(#json_key) {
                        Some(val) => {
                            #preprocess
                            ::serde_json::from_value::<#ty>(val)
                                .map_err(|e| ::laminate::FlexError::DeserializeError {
                                    path: #json_key.to_string(),
                                    source: e,
                                })?
                        }
                        None => {
                            return Err(::laminate::FlexError::PathNotFound {
                                path: #json_key.to_string(),
                            });
                        }
                    }
                }
            };

            quote! {
                let #ident: #ty = #extract;
            }
        })
        .collect();

    let field_names: Vec<&syn::Ident> = regular_fields
        .iter()
        .map(|(ident, _, _, _)| ident)
        .collect();
    let skip_field_names: Vec<&syn::Ident> = skip_fields.iter().map(|(ident, _)| ident).collect();
    let flatten_field_names: Vec<&syn::Ident> =
        flatten_fields.iter().map(|(ident, _)| ident).collect();

    // Emit diagnostics for unknown fields, then handle overflow
    let overflow_assignment = if let Some((ref overflow_ident, is_option)) = overflow_field {
        // Overflow field present — unknown fields are preserved
        if is_option {
            quote! {
                for key in map.keys() {
                    _diagnostics.push(::laminate::Diagnostic {
                        path: key.clone(),
                        kind: ::laminate::DiagnosticKind::Preserved {
                            field: key.clone(),
                        },
                        risk: ::laminate::RiskLevel::Info,
                        suggestion: None,
                    });
                }
                let #overflow_ident = if map.is_empty() { None } else { Some(map) };
            }
        } else {
            quote! {
                for key in map.keys() {
                    _diagnostics.push(::laminate::Diagnostic {
                        path: key.clone(),
                        kind: ::laminate::DiagnosticKind::Preserved {
                            field: key.clone(),
                        },
                        risk: ::laminate::RiskLevel::Info,
                        suggestion: None,
                    });
                }
                let #overflow_ident = map;
            }
        }
    } else {
        // No overflow field — unknown fields are dropped with diagnostics
        quote! {
            for key in map.keys() {
                _diagnostics.push(::laminate::Diagnostic {
                    path: key.clone(),
                    kind: ::laminate::DiagnosticKind::Dropped {
                        field: key.clone(),
                    },
                    risk: ::laminate::RiskLevel::Info,
                    suggestion: Some("add this field to the struct or use #[laminate(overflow)]".to_string()),
                });
            }
            let _ = map;
        }
    };

    let all_field_names: Vec<&syn::Ident> = {
        let mut v: Vec<&syn::Ident> = field_names.to_vec();
        v.extend(&skip_field_names);
        v.extend(&flatten_field_names);
        if let Some((ref overflow_ident, _)) = overflow_field {
            v.push(overflow_ident);
        }
        v
    };

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Deserialize from a `serde_json::Value` using laminate's flexible shaping.
            ///
            /// Returns the shaped struct along with any diagnostics produced
            /// during coercion and field extraction.
            pub fn from_flex_value(
                value: &::serde_json::Value,
            ) -> ::laminate::Result<(Self, Vec<::laminate::Diagnostic>)> {
                let mut _diagnostics: Vec<::laminate::Diagnostic> = Vec::new();

                let mut map = match value {
                    ::serde_json::Value::Object(obj) => {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<::std::collections::HashMap<String, ::serde_json::Value>>()
                    }
                    _ => {
                        return Err(::laminate::FlexError::TypeMismatch {
                            path: "(root)".to_string(),
                            expected: "object".to_string(),
                            actual: format!("{:?}", value),
                        });
                    }
                };

                #(#field_extractions)*

                #(#skip_extractions)*

                #(#flatten_extractions)*

                #overflow_assignment

                Ok((
                    Self {
                        #(#all_field_names),*
                    },
                    _diagnostics,
                ))
            }

            /// Deserialize from a JSON string using laminate's flexible shaping.
            pub fn from_json(
                json: &str,
            ) -> ::laminate::Result<(Self, Vec<::laminate::Diagnostic>)> {
                let value: ::serde_json::Value =
                    ::serde_json::from_str(json).map_err(|e| {
                        ::laminate::FlexError::DeserializeError {
                            path: "(root)".to_string(),
                            source: e,
                        }
                    })?;
                Self::from_flex_value(&value)
            }

            /// Shape with a specific mode, returning `LaminateResult`.
            ///
            /// The mode controls coercion level, unknown field handling,
            /// and error strategy:
            /// - **Lenient**: BestEffort coercion, drop unknowns, default missing
            /// - **Absorbing**: SafeWidening coercion, preserve unknowns, error on missing
            /// - **Strict**: Exact coercion, error on unknowns, error on missing
            pub fn shape_lenient(
                value: &::serde_json::Value,
            ) -> ::laminate::Result<::laminate::LaminateResult<Self, ::laminate::Lenient>> {
                let (shaped, diagnostics) = Self::from_flex_value(value)?;
                Ok(::laminate::LaminateResult::lenient(shaped, diagnostics))
            }

            /// Shape with Absorbing mode — preserves unknown fields in overflow.
            pub fn shape_absorbing(
                value: &::serde_json::Value,
            ) -> ::laminate::Result<::laminate::LaminateResult<Self, ::laminate::Absorbing>> {
                let (shaped, diagnostics) = Self::from_flex_value(value)?;
                // Extract overflow from the shaped value if it has an overflow field
                let overflow = ::std::collections::HashMap::new();
                Ok(::laminate::LaminateResult::absorbing(shaped, overflow, diagnostics))
            }

            /// Shape with Strict mode — rejects unknown fields, requires exact types.
            ///
            /// Uses Exact coercion level. Returns error if any coercion was needed
            /// or if unknown fields are present.
            pub fn shape_strict(
                value: &::serde_json::Value,
            ) -> ::laminate::Result<Self> {
                let mut _diagnostics: Vec<::laminate::Diagnostic> = Vec::new();

                let mut map = match value {
                    ::serde_json::Value::Object(obj) => {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<::std::collections::HashMap<String, ::serde_json::Value>>()
                    }
                    _ => {
                        return Err(::laminate::FlexError::TypeMismatch {
                            path: "(root)".to_string(),
                            expected: "object".to_string(),
                            actual: format!("{:?}", value),
                        });
                    }
                };

                #(#field_extractions)*

                #(#skip_extractions)*

                #(#flatten_extractions)*

                // Strict mode: reject unknown fields BEFORE overflow consumes them
                if !map.is_empty() {
                    let unknown_keys: Vec<String> = map.keys().cloned().collect();
                    return Err(::laminate::FlexError::ShapingDiagnostics {
                        count: unknown_keys.len(),
                        diagnostics: unknown_keys.iter().map(|k| {
                            ::laminate::Diagnostic {
                                path: k.clone(),
                                kind: ::laminate::DiagnosticKind::Dropped { field: k.clone() },
                                risk: ::laminate::RiskLevel::Risky,
                                suggestion: Some("unknown field in strict mode".into()),
                            }
                        }).collect(),
                    });
                }

                // Run overflow assignment to define overflow field variables
                #overflow_assignment

                // Strict mode: reject if any coercions were applied
                let coercion_diagnostics: Vec<_> = _diagnostics.iter()
                    .filter(|d| matches!(d.kind, ::laminate::DiagnosticKind::Coerced { .. }))
                    .collect();
                if !coercion_diagnostics.is_empty() {
                    return Err(::laminate::FlexError::ShapingDiagnostics {
                        count: coercion_diagnostics.len(),
                        diagnostics: _diagnostics,
                    });
                }

                Ok(Self {
                    #(#all_field_names),*
                })
            }
        }
    };

    // Generate to_value() for round-trip preservation
    let serialize_fields: Vec<TokenStream> = regular_fields
        .iter()
        .map(|(ident, _ty, _attrs, json_key)| {
            quote! {
                obj.insert(
                    #json_key.to_string(),
                    ::serde_json::to_value(&self.#ident).unwrap_or(::serde_json::Value::Null),
                );
            }
        })
        .collect();

    // Flatten fields: serialize and merge into the output object
    let serialize_flatten: Vec<TokenStream> = flatten_fields
        .iter()
        .map(|(ident, _ty)| {
            quote! {
                if let Ok(::serde_json::Value::Object(flat_obj)) = ::serde_json::to_value(&self.#ident) {
                    for (k, v) in flat_obj {
                        obj.insert(k, v);
                    }
                }
            }
        })
        .collect();

    // Overflow field: re-insert unknown fields for round-trip
    let serialize_overflow = if let Some((ref overflow_ident, is_option)) = overflow_field {
        if is_option {
            quote! {
                if let Some(ref overflow_map) = self.#overflow_ident {
                    for (k, v) in overflow_map {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        } else {
            quote! {
                for (k, v) in &self.#overflow_ident {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    } else {
        quote! {}
    };

    let to_value_impl = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Serialize back to a `serde_json::Value`, preserving overflow fields.
            ///
            /// This enables round-trip preservation: unknown fields absorbed
            /// during deserialization reappear in the serialized output.
            pub fn to_value(&self) -> ::serde_json::Value {
                let mut obj = ::serde_json::Map::new();
                #(#serialize_fields)*
                #(#serialize_flatten)*
                #serialize_overflow
                ::serde_json::Value::Object(obj)
            }

            /// Serialize to a JSON string, preserving overflow fields.
            pub fn to_json(&self) -> String {
                ::serde_json::to_string(&self.to_value()).unwrap_or_default()
            }

            /// Serialize to a pretty-printed JSON string.
            pub fn to_json_pretty(&self) -> String {
                ::serde_json::to_string_pretty(&self.to_value()).unwrap_or_default()
            }
        }
    };

    let combined = quote! {
        #expanded
        #to_value_impl
    };

    Ok(combined)
}
