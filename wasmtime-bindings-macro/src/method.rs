use crate::attr::TransformAttributes;
use crate::signature::{read_signature, MethodSignature, ParameterType, Return};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{self, Ident, ItemFn};

pub(crate) fn need_context(sig: &MethodSignature) -> bool {
    for p in &sig.params {
        if let ParameterType::Ptr(_) = p.ty {
            return true;
        }
        if let ParameterType::Context(_) = p.ty {
            return true;
        }
    }
    if let Some(Return::Ptr(_)) = sig.result {
        return true;
    }
    false
}

pub(crate) fn transform_sig(
    sig: &MethodSignature,
    context_name: TokenStream,
    wasmtime_bindings_common: TokenStream,
) -> (
    TokenStream,
    TokenStream,
    TokenStream,
    TokenStream,
    TokenStream,
    TokenStream,
) {
    let mut abi_params = TokenStream::new();
    let mut params_conversion = TokenStream::new();
    let mut call_args = TokenStream::new();
    let mut sig_build = TokenStream::new();
    abi_params.extend(quote! {
        vmctx: *mut VMContext,
    });
    // TODO fix CallConv assumption below
    sig_build.extend(quote! {
        let mut sig = ir::Signature::new(isa::CallConv::SystemV);
        sig.params.push(ir::AbiParam::special(
            ir::types::I64,
            ir::ArgumentPurpose::VMContext,
        ));
    });
    for (i, p) in sig.params.iter().enumerate() {
        let id = p.id;
        let internal_id = Ident::new(&format!("_a{}", i), Span::call_site());
        match p.ty {
            ParameterType::VMContextMutPtr => {
                call_args.extend(quote! { vmctx, });
            }
            ParameterType::Context(_) => {
                call_args.extend(quote! { _ctx, });
            }
            ParameterType::Ptr(_) => {
                abi_params.extend(quote! {
                    #internal_id: <#context_name as #wasmtime_bindings_common :: WasmMem>::Abi,
                });
                params_conversion.extend(quote! {
                    let #id = _ctx.as_ptr(#internal_id);
                });
                call_args.extend(quote! { #id , });
                sig_build.extend(quote! {
                    sig.params.push(ir::AbiParam::new(
                        #wasmtime_bindings_common :: get_ir_type::<<#context_name as #wasmtime_bindings_common :: WasmMem>::Abi>()
                    ));
                });
            }
            ParameterType::Simple(ty, _) => {
                abi_params.extend(quote! {
                    #internal_id: <#ty as #wasmtime_bindings_common :: AbiParam>::Abi,
                });
                params_conversion.extend(quote! {
                    let #id = <#ty as #wasmtime_bindings_common :: AbiParam>::create_from_abi(#internal_id);
                });
                call_args.extend(quote! { #id , });
                sig_build.extend(quote! {
                    sig.params.push(ir::AbiParam::new(
                        #wasmtime_bindings_common :: get_ir_type::<<#ty as #wasmtime_bindings_common :: AbiParam>::Abi>()
                    ));
                });
            }
            ParameterType::SelfRef(ref r) => {
                params_conversion.extend(if r.mutable() {
                    quote! { let _self = get_self_mut(vmctx); }
                } else {
                    quote! { let _self = get_self(vmctx); }
                });
            }
        }
    }
    let (abi_return, ret_conversion, sig_return) = match sig.result {
        Some(Return::Ptr(_)) => (
            quote! {
                -> <#context_name as #wasmtime_bindings_common :: WasmMem>::Abi
            },
            quote! {
                _ctx.as_off(_res)
            },
            quote! {
                sig.returns.push(ir::AbiParam::new(
                    #wasmtime_bindings_common :: get_ir_type::<<#context_name as #wasmtime_bindings_common :: WasmMem>::Abi>()
                ));
            },
        ),
        Some(Return::Simple(ty)) => (
            quote! {
                -> <#ty as #wasmtime_bindings_common :: AbiRet>::Abi
            },
            quote! {
            _res.convert_to_abi()
            },
            quote! {
                sig.returns.push(ir::AbiParam::new(
                    #wasmtime_bindings_common :: get_ir_type::<<#ty as #wasmtime_bindings_common :: AbiRet>::Abi>()
                ));
            },
        ),
        None => (TokenStream::new(), TokenStream::new(), TokenStream::new()),
    };
    sig_build.extend(sig_return);

    (
        abi_params,
        abi_return,
        params_conversion,
        ret_conversion,
        call_args,
        sig_build,
    )
}

pub(crate) fn wrap_method(f: ItemFn, attr: TransformAttributes) -> TokenStream {
    let sig = &f.sig;
    let name = &sig.ident;
    let vis = &f.vis;
    assert!(sig.constness.is_none());
    assert!(sig.asyncness.is_none());
    assert!(sig.unsafety.is_none());
    assert!(sig.abi.is_none());
    //assert!(sig.generics)
    let inputs = &sig.inputs;
    assert!(sig.variadic.is_none());
    let output = &sig.output;

    let rsig = read_signature(sig, &attr.context);

    let body = &f.block;

    let wasmtime_bindings_common = quote! { :: wasmtime_bindings_common };
    let (build_context, context_name) = if need_context(&rsig) {
        if let Some(context_name) = attr.context {
            (
                quote! {
                    let _ctx = #context_name :: from_vmctx(vmctx);
                },
                quote! { #context_name },
            )
        } else {
            (
                quote! {
                    let _ctx = #wasmtime_bindings_common :: VMContextWrapper(vmctx);
                },
                quote! { #wasmtime_bindings_common :: VMContextWrapper },
            )
        }
    } else {
        (quote! { panic!() }, quote! { panic!() })
    };

    let (abi_params, abi_return, params_conversion, ret_conversion, call_args, sig_build) =
        transform_sig(&rsig, context_name, wasmtime_bindings_common.clone());

    let def_module = if let Some(mod_name) = attr.module {
        quote! {
            #vis mod #mod_name {
                use super::*;
                use #wasmtime_bindings_common :: codegen :: {ir, isa};
                pub fn signature() -> ir::Signature {
                    #sig_build
                    sig
                }
            }
        }
    } else {
        quote! {}
    };

    let result = quote! {
    pub extern fn #name (#abi_params) #abi_return {
        let _closure = | #inputs | #output #body;
        #build_context
        #params_conversion
        let _res = _closure( #call_args );
        #ret_conversion
    }
    #def_module
        };

    result.into()
}
