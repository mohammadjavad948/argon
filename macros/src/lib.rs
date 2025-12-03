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
                
                openapi_path_functions.push(quote! {
                    #[doc = concat!("Auto-generated utoipa path wrapper for ", #struct_name_str, "::", #fn_name_str)]
                    #[doc = concat!("This function is only for OpenAPI documentation generation.")]
                    #[doc = concat!("The actual handler is ", #struct_name_str, "::", #fn_name_str)]
                    #[utoipa::path(
                        #utoipa_method,
                        path = #path_lit,
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
    
    // Collect wrapper function names for the OpenAPI paths
    let mut openapi_path_names = Vec::new();
    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            if extract_route_attr(&method.attrs).is_some() {
                let fn_name = &method.sig.ident;
                let wrapper_name = format_ident!("__utoipa_path_{}", fn_name);
                openapi_path_names.push(wrapper_name);
            }
        }
    }

    // Generate the router function and OpenAPI struct
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
        #[derive(utoipa::OpenApi)]
        #[openapi(
            paths(
                #(#openapi_path_names),*
            )
        )]
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
