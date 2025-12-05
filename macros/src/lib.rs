use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, FnArg, ImplItem, ItemImpl, LitStr, Meta, Type, LitInt};

/// Macro that generates an Axum router from struct methods with route attributes
///
/// Usage:
/// ```rust
/// struct MyController;
///
/// #[controller]
/// impl MyController {
///     #[get("/users")]
///     async fn get_users() -> String {
///         "users".to_string()
///     }
///     
///     #[post("/users")]
///     async fn create_user() -> String {
///         "created".to_string()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(_args: TokenStream, input: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(input as ItemImpl);
    let self_ty = &impl_block.self_ty;
    let struct_name = match &**self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.last().map(|s| &s.ident).unwrap(),
        _ => {
            return syn::Error::new(impl_block.span(), "Expected a struct type")
                .to_compile_error()
                .into();
        }
    };

    let mut route_registrations = Vec::new();
    let mut openapi_path_functions = Vec::new();

    // Iterate through items in the impl block
    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            // Check for route attributes
            if let Some((method_name, path)) = extract_route_attr(&method.attrs) {
                let fn_name = &method.sig.ident;

                // Determine if method takes &self, &mut self, or no self
                let has_self = method
                    .sig
                    .inputs
                    .iter()
                    .any(|input| matches!(input, FnArg::Receiver(_)));

                let handler_call = if has_self {
                    // Method with self
                    quote! {
                        #struct_name::#fn_name
                    }
                } else {
                    // Associated function
                    quote! {
                        #struct_name::#fn_name
                    }
                };

                // Generate route registration based on HTTP method
                let axum_method = format_ident!("{}", method_name);
                route_registrations.push(quote! {
                    router = router.route(#path, axum::routing::#axum_method(#handler_call));
                });

                // Create a wrapper function name for utoipa path documentation
                // This function will be created outside the impl block with #[utoipa::path]
                let utoipa_wrapper_name = format_ident!("__utoipa_path_{}", fn_name);
                let utoipa_method = format_ident!("{}", method_name);
                let path_str = path.clone();
                
                // Extract the function signature parts
                let fn_vis = &method.vis;
                let fn_async = method.sig.asyncness;
                let fn_inputs = &method.sig.inputs;
                let fn_output = &method.sig.output;
                let fn_generics = &method.sig.generics;
                let fn_where_clause = &method.sig.generics.where_clause;
                
                // Generate a wrapper function with utoipa::path attribute outside the impl block
                // The wrapper has the same signature as the original but is just for documentation
                // Remove leading slash from path since it will be nested under "/" in MainApiDoc
                let path_for_utoipa = if path_str.starts_with('/') {
                    &path_str[1..]
                } else {
                    &path_str
                };
                let path_lit = syn::LitStr::new(path_for_utoipa, method.span());
                
                let struct_name_str = struct_name.to_string();
                let fn_name_str = fn_name.to_string();
                
                // Extract all utoipa_response attributes (supports multiple)
                let response_attrs = extract_utoipa_response_attrs(&method.attrs);
                
                // Build the utoipa::path attribute with optional responses
                let mut path_attr_tokens = quote! {
                    #utoipa_method,
                    path = #path_lit,
                };
                
                if !response_attrs.is_empty() {
                    path_attr_tokens = quote! {
                        #utoipa_method,
                        path = #path_lit,
                        responses(
                            #(#response_attrs),*
                        ),
                    };
                }
                
                openapi_path_functions.push(quote! {
                    #[doc = concat!("Auto-generated utoipa path wrapper for ", #struct_name_str, "::", #fn_name_str)]
                    #[doc = concat!("This function is only for OpenAPI documentation generation.")]
                    #[doc = concat!("The actual handler is ", #struct_name_str, "::", #fn_name_str)]
                    #[utoipa::path(
                        #path_attr_tokens
                    )]
                    #fn_vis #fn_async fn #utoipa_wrapper_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause {
                        // This function is only for OpenAPI documentation generation
                        // The actual handler is #struct_name::#fn_name
                        // This body will never be executed
                        unimplemented!("This is a documentation-only wrapper function")
                    }
                });
            }
        }
    }

    // Create a name for the generated OpenAPI struct: "MyController" -> "MyControllerApi"
    let api_struct_name = format_ident!("{}Api", struct_name);
    
    // Collect wrapper function names for the OpenAPI paths and extract schema types
    let mut openapi_path_names = Vec::new();
    let mut schema_types = Vec::new();
    
    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            if extract_route_attr(&method.attrs).is_some() {
                let fn_name = &method.sig.ident;
                let wrapper_name = format_ident!("__utoipa_path_{}", fn_name);
                openapi_path_names.push(wrapper_name);
                
                // Extract schema types from utoipa_response attributes
                let response_types = extract_response_schema_types(&method.attrs);
                schema_types.extend(response_types);
            }
        }
    }
    
    // Remove duplicates from schema_types (comparing by string representation)
    let mut unique_schemas = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for schema_type in schema_types {
        let type_str = quote!(#schema_type).to_string();
        if !seen.contains(&type_str) {
            seen.insert(type_str);
            unique_schemas.push(schema_type);
        }
    }

    // Generate the router function and OpenAPI struct
    // Conditionally include components section if we have schemas
    let openapi_attr = if unique_schemas.is_empty() {
        quote! {
            #[derive(utoipa::OpenApi)]
            #[openapi(
                paths(
                    #(#openapi_path_names),*
                )
            )]
        }
    } else {
        quote! {
            #[derive(utoipa::OpenApi)]
            #[openapi(
                paths(
                    #(#openapi_path_names),*
                ),
                components(schemas(
                    #(#unique_schemas),*
                ))
            )]
        }
    };
    
    let expanded = quote! {
        // The original impl block
        #impl_block

        impl argon_core::controller::Controller for #self_ty {
            /// Generates an Axum router from the controller methods
            fn router() -> axum::Router {
                use axum::Router;

                let mut router = Router::new();

                #(#route_registrations)*

                router
            }
        }

        // Auto-generated utoipa path wrapper functions (must be at module level)
        #(#openapi_path_functions)*

        // Auto-generated OpenAPI struct
        // This creates a struct that lists all the paths found in this controller.
        // You can nest this into your main ApiDoc.
        #openapi_attr
        pub struct #api_struct_name;
    };

    TokenStream::from(expanded)
}

/// Extract route information from attributes
/// Looks for route macro attributes like #[get("/path")] or #[argon_macros::get("/path")]
/// Note: This will only work if the attributes haven't been consumed by attribute macros yet
fn extract_route_attr(attrs: &[Attribute]) -> Option<(String, String)> {
    for attr in attrs {
        // Check if this is one of our route macros
        let path_segments: Vec<_> = attr.path().segments.iter().collect();
        if path_segments.is_empty() {
            continue;
        }

        // Get the last segment (handles both #[get("/path")] and #[argon_macros::get("/path")])
        let last_segment = path_segments.last().unwrap();
        let method = last_segment.ident.to_string().to_lowercase();
        if matches!(method.as_str(), "get" | "post" | "put" | "delete" | "patch") {
            // Try to parse as a list meta (e.g., #[get("/path")])
            if let Meta::List(meta) = &attr.meta {
                // Extract the path from the tokens - it should be a string literal
                let tokens = meta.tokens.clone();
                if let Ok(path_lit) = syn::parse2::<LitStr>(tokens) {
                    return Some((method, path_lit.value()));
                }
            }
        }
    }
    None
}

/// Extract all utoipa_response attribute information
/// Supports multiple attributes for multiple status codes:
/// - #[utoipa_response(Type)] - simple form, defaults to status 200 with body
/// - #[utoipa_response(response = Type)] - use Type as IntoResponses (just the type name)
/// - #[utoipa_response(status = 200, body = Type)] - with explicit status
/// - #[utoipa_response(status = 200, body = Type, description = "Success")] - with description
/// 
/// Example with multiple responses:
/// ```rust
/// #[get("/users/{id}")]
/// #[utoipa_response(status = 200, body = User, description = "User found")]
/// #[utoipa_response(status = 404, body = Error, description = "User not found")]
/// #[utoipa_response(status = 500, body = Error, description = "Internal server error")]
/// async fn get_user() -> Result<User, Error> { ... }
/// ```
/// 
/// Returns a vector of response tokens to be inserted into the utoipa::path attribute
fn extract_utoipa_response_attrs(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
    let mut responses = Vec::new();
    
    for attr in attrs {
        let path_segments: Vec<_> = attr.path().segments.iter().collect();
        if path_segments.is_empty() {
            continue;
        }

        // Get the last segment (handles both #[utoipa_response(...)] and #[argon_macros::utoipa_response(...)])
        let last_segment = path_segments.last().unwrap();
        if last_segment.ident == "utoipa_response" {
            if let Meta::List(meta) = &attr.meta {
                let tokens = meta.tokens.clone();
                
                // Try to parse as named arguments first (e.g., #[utoipa_response(response = UserResponse)])
                if let Ok(parsed) = syn::parse2::<UtoipaResponseArgs>(tokens.clone()) {
                    // If response is specified, use it as IntoResponses (just the type name)
                    if let Some(response_type) = parsed.response {
                        responses.push(quote! {
                            #response_type
                        });
                        continue;
                    }
                    
                    // Otherwise, use body with status/description
                    if let Some(body_type) = parsed.body {
                        let status = parsed.status.unwrap_or(200);
                        let description = parsed.description.as_deref().unwrap_or("Success");
                        
                        responses.push(quote! {
                            (status = #status, description = #description, body = #body_type)
                        });
                        continue;
                    }
                }
                
                // Try to parse as a simple type (e.g., #[utoipa_response(Pet)])
                // This defaults to body type for backward compatibility
                if let Ok(response_type) = syn::parse2::<Type>(tokens) {
                    // Simple form: just a type, default to status 200 with body
                    responses.push(quote! {
                        (status = 200, description = "Success", body = #response_type)
                    });
                }
            }
        }
    }
    
    responses
}

/// Extract schema types from utoipa_response attributes
/// Returns a vector of types that should be included in components(schemas(...))
fn extract_response_schema_types(attrs: &[Attribute]) -> Vec<Type> {
    let mut schema_types = Vec::new();
    
    for attr in attrs {
        let path_segments: Vec<_> = attr.path().segments.iter().collect();
        if path_segments.is_empty() {
            continue;
        }

        let last_segment = path_segments.last().unwrap();
        if last_segment.ident == "utoipa_response" {
            if let Meta::List(meta) = &attr.meta {
                let tokens = meta.tokens.clone();
                
                // Try to parse as named arguments
                if let Ok(parsed) = syn::parse2::<UtoipaResponseArgs>(tokens.clone()) {
                    // Add body type if present
                    if let Some(body_type) = parsed.body {
                        schema_types.push(body_type);
                    }
                    
                    // For response types (IntoResponses), extract generic parameters
                    // Note: Type aliases won't be resolved here, but utoipa should handle them
                    if let Some(response_type) = parsed.response {
                        // Extract types from generic parameters (if it's a generic type, not a type alias)
                        extract_types_from_generic(&response_type, &mut schema_types);
                        // Don't add the response type itself if it's likely a type alias or enum
                        // Utoipa will handle IntoResponses types automatically
                    }
                    continue;
                }
                
                // Try to parse as a simple type
                if let Ok(response_type) = syn::parse2::<Type>(tokens) {
                    schema_types.push(response_type);
                }
            }
        }
    }
    
    schema_types
}

/// Recursively extract types from generic type parameters
/// For example, CoreResponse<T, N, U, I> would extract T, N, U, I
fn extract_types_from_generic(ty: &Type, schema_types: &mut Vec<Type>) {
    match ty {
        Type::Path(type_path) => {
            if let Some(path_segment) = type_path.path.segments.last() {
                match &path_segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        for arg in &args.args {
                            match arg {
                                syn::GenericArgument::Type(ty) => {
                                    // Recursively extract from nested generics
                                    extract_types_from_generic(ty, schema_types);
                                    // Add the type itself
                                    schema_types.push(ty.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// Helper struct to parse utoipa_response attribute arguments
#[derive(Debug)]
struct UtoipaResponseArgs {
    status: Option<u16>,
    body: Option<Type>,
    response: Option<Type>,
    description: Option<String>,
}

impl syn::parse::Parse for UtoipaResponseArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut status = None;
        let mut body = None;
        let mut response = None;
        let mut description = None;
        
        // Parse comma-separated key-value pairs
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            
            if key_str == "status" {
                let _eq: syn::Token![=] = input.parse()?;
                let lit: LitInt = input.parse()?;
                status = Some(lit.base10_parse::<u16>()?);
            } else if key_str == "body" {
                let _eq: syn::Token![=] = input.parse()?;
                body = Some(input.parse()?);
            } else if key_str == "response" {
                let _eq: syn::Token![=] = input.parse()?;
                response = Some(input.parse()?);
            } else if key_str == "description" {
                let _eq: syn::Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                description = Some(lit.value());
            } else {
                return Err(syn::Error::new(key.span(), format!("Unknown argument: {}", key_str)));
            }
            
            // Check for comma
            if !input.is_empty() {
                let _comma: syn::Token![,] = input.parse()?;
            }
        }
        
        // Either body or response must be specified, but not both
        if body.is_some() && response.is_some() {
            return Err(input.error("Cannot specify both 'body' and 'response'. Use 'body' for simple types or 'response' for IntoResponses types."));
        }
        
        Ok(UtoipaResponseArgs {
            status,
            body,
            response,
            description,
        })
    }
}

/// Macro for GET route
#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
    route_attr_macro("get", args, input)
}

/// Macro for POST route
#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
    route_attr_macro("post", args, input)
}

/// Macro for PUT route
#[proc_macro_attribute]
pub fn put(args: TokenStream, input: TokenStream) -> TokenStream {
    route_attr_macro("put", args, input)
}

/// Macro for DELETE route
#[proc_macro_attribute]
pub fn delete(args: TokenStream, input: TokenStream) -> TokenStream {
    route_attr_macro("delete", args, input)
}

/// Macro for PATCH route
#[proc_macro_attribute]
pub fn patch(args: TokenStream, input: TokenStream) -> TokenStream {
    route_attr_macro("patch", args, input)
}

/// Attribute macro for specifying utoipa response documentation
/// 
/// You can chain multiple `#[utoipa_response]` attributes to specify multiple status codes.
/// 
/// Usage:
/// ```rust
/// // Simple type (defaults to status 200)
/// #[get("/users")]
/// #[utoipa_response(User)]
/// async fn get_users() -> String { ... }
/// 
/// // Simple type with explicit status
/// #[post("/users")]
/// #[utoipa_response(status = 201, body = User, description = "User created")]
/// async fn create_user() -> String { ... }
/// 
/// // Multiple status codes
/// #[get("/users/{id}")]
/// #[utoipa_response(status = 200, body = User, description = "User found")]
/// #[utoipa_response(status = 404, body = Error, description = "User not found")]
/// #[utoipa_response(status = 500, body = Error, description = "Internal server error")]
/// async fn get_user() -> Result<User, Error> { ... }
/// 
/// // IntoResponses type (like UserResponse<T, N, U, I>)
/// #[get("/users/{id}")]
/// #[utoipa_response(response = UserResponse<User, NotFound, Unauthorized, InternalError>)]
/// async fn get_user() -> UserResponse<...> { ... }
/// 
/// // Mix of IntoResponses and individual responses
/// #[get("/users/{id}")]
/// #[utoipa_response(response = UserResponse<User, NotFound, Unauthorized, InternalError>)]
/// #[utoipa_response(status = 503, body = Error, description = "Service unavailable")]
/// async fn get_user() -> Result<UserResponse<...>, Error> { ... }
/// ```
/// 
/// This attribute is consumed by the `#[controller]` macro to generate
/// OpenAPI documentation. It's a pass-through macro that doesn't modify the function.
#[proc_macro_attribute]
pub fn utoipa_response(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Pass through - the controller macro will read this attribute
    input
}

/// Helper function for route attribute macros
/// These macros are pass-through - they don't modify the function
/// The router macro will read the original attributes before these macros process them
/// However, since attribute macros consume their attribute, we need a different approach.
/// We'll store the route info in a way that the router macro can find it.
///
/// Actually, the router macro runs on the impl block and can see the method attributes
/// before they're processed. So we just need to make these pass-through.
fn route_attr_macro(_method: &str, _args: TokenStream, input: TokenStream) -> TokenStream {
    // For now, just pass through - the router macro should see the original attribute
    // But this won't work because attribute macros consume the attribute...
    // So we need to preserve the info somehow.
    // Let's add it as a doc attribute that the router can parse
    input
}
