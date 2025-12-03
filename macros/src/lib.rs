use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, FnArg, ImplItem, ItemImpl, LitStr, Meta};

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
    let mut impl_block = parse_macro_input!(input as ItemImpl);
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
    let mut openapi_function_names = Vec::new();

    // Iterate through items in the impl block (mutably so we can add attributes)
    for item in &mut impl_block.items {
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

                // Generate utoipa::path attribute
                let utoipa_method = format_ident!("{}", method_name);
                let path_str = path.clone();
                
                // Build the attribute tokens - parse as a full attribute by wrapping in a dummy item
                let attr_tokens = quote! {
                    #[utoipa::path(
                        #utoipa_method,
                        path = #path_str,
                    )]
                    fn dummy() {}
                };
                
                // Parse as an ItemFn and extract the attribute
                match syn::parse2::<syn::ItemFn>(attr_tokens) {
                    Ok(item_fn) => {
                        // Extract the first attribute (our utoipa::path attribute)
                        if let Some(attr) = item_fn.attrs.first().cloned() {
                            method.attrs.push(attr);
                        }
                    }
                    Err(e) => {
                        return syn::Error::new(
                            method.span(),
                            format!("Failed to parse utoipa attribute: {}", e),
                        )
                        .to_compile_error()
                        .into();
                    }
                }

                // Save the function name to register in the OpenApi struct later
                openapi_function_names.push(fn_name.clone());
            }
        }
    }

    // Create a name for the generated OpenAPI struct: "MyController" -> "MyControllerApi"
    let api_struct_name = format_ident!("{}Api", struct_name);

    // Generate the router function and OpenAPI struct
    let expanded = quote! {
        // The modified impl block (now containing #[utoipa::path] attributes)
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

        // Auto-generated OpenAPI struct
        // This creates a struct that lists all the paths found in this controller.
        // You can nest this into your main ApiDoc.
        #[derive(utoipa::OpenApi)]
        #[openapi(
            paths(
                #(
                    #struct_name::#openapi_function_names
                ),*
            )
        )]
        pub struct #api_struct_name;
    };

    TokenStream::from(expanded)
}

/// Extract route information from attributes
/// Looks for route macro attributes like #[get("/path")]
/// Note: This will only work if the attributes haven't been consumed by attribute macros yet
fn extract_route_attr(attrs: &[Attribute]) -> Option<(String, String)> {
    for attr in attrs {
        // Check if this is one of our route macros
        let path_segments: Vec<_> = attr.path().segments.iter().collect();
        if path_segments.is_empty() {
            continue;
        }

        let method = path_segments[0].ident.to_string().to_lowercase();
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
