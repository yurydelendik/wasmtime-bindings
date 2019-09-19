use crate::attr::TransformAttributes;
use crate::method::{need_context, transform_sig};
use crate::signature::{read_signature, Parameter, ParameterType};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemTrait, TraitItem, TraitItemMethod};

fn generate_method_wrapper(
    m: &TraitItemMethod,
    wasm_bindings_common: TokenStream,
    attr: &TransformAttributes,
) -> (TokenStream, TokenStream) {
    let rsig = read_signature(&m.sig, &attr.context);
    let _self_ref = match rsig.params.get(0) {
        Some(Parameter {
            ty: ParameterType::SelfRef(ref sr),
            ..
        }) => sr,
        _ => {
            panic!("expected first parameter to be self ref");
        }
    };

    let (build_context, context_name) = if need_context(&rsig) {
        if let Some(context_name) = &attr.context {
            (
                quote! {
                    let _ctx = #context_name :: from_vmctx(vmctx);
                },
                quote! { #context_name },
            )
        } else {
            (
                quote! {
                    let _ctx = #wasm_bindings_common :: VMContextWrapper(vmctx);
                },
                quote! { #wasm_bindings_common :: VMContextWrapper },
            )
        }
    } else {
        (quote! { panic!() }, quote! { panic!() })
    };

    let (abi_params, abi_return, params_conversion, ret_conversion, call_args, sig_build) =
        transform_sig(&rsig, context_name, wasm_bindings_common);
    let name = &m.sig.ident;
    let result = quote! {
        pub extern fn #name (#abi_params) #abi_return {
            #build_context
            #params_conversion
            let _res = _self . #name ( #call_args );
            #ret_conversion
        }
    };
    let sig_build = quote! {
        pub fn #name () -> ir::Signature {
            #sig_build
            sig
        }
    };
    (result, sig_build)
}

pub(crate) fn wrap_trait(tr: ItemTrait, attr: TransformAttributes) -> TokenStream {
    let vis = &tr.vis;
    let ident = &tr.ident;

    let mod_name = attr.module.as_ref().unwrap();
    let wasmtime_bindings_common = quote! { :: wasmtime_bindings_common };
    let mut mod_wrappers = TokenStream::new();
    let mut signatures = TokenStream::new();
    for i in &tr.items {
        if let TraitItem::Method(ref m) = i {
            let (wrapper, signature) =
                generate_method_wrapper(m, wasmtime_bindings_common.clone(), &attr);
            mod_wrappers.extend(wrapper);
            signatures.extend(signature);
        }
    }

    let mod_content = quote! {
        #vis mod #mod_name {
            use super::*;
            use #wasmtime_bindings_common :: {VMContext, AbiParam, AbiRet, WasmMem};
            use ::std::boxed::Box;
            use ::std::cell::{Ref, RefMut, RefCell};
            type Subject = dyn super :: #ident;

            pub struct State {
                pub subject: RefCell<
                    Box<dyn super :: #ident + 'static>
                >,
            }
            impl State {
                fn from<'a>(vmctx: *mut VMContext) -> &'a mut Self {
                    unsafe { &mut *(&mut *vmctx).host_state().downcast_mut::<Self>().unwrap() }
                }
            }
            #vis fn get_self(vmctx: *mut VMContext) -> Ref<'static, Subject> {
                use ::core::ops::Deref;
                Ref::map(State::from(vmctx).subject.borrow(), |b| b.deref())
            }
            #vis fn get_self_mut(vmctx: *mut VMContext) -> RefMut<'static, Subject> {
                use ::core::ops::DerefMut;
                RefMut::map(State::from(vmctx).subject.borrow_mut(), |b| b.deref_mut())
            }
            #mod_wrappers

            pub mod signatures {
                use super::*;
                use #wasmtime_bindings_common :: codegen :: {ir, isa};
                #signatures
            }
        }
    };
    quote! {
        #tr

        #mod_content
    }
}
