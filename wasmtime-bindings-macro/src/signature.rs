use syn::{token, FnArg, Ident, Pat, Path, ReturnType, Signature, Type};

#[derive(Clone, Copy)]
pub(crate) struct RefFlag(Option<token::Mut>);

impl RefFlag {
    pub(crate) fn mutable(&self) -> bool {
        self.0.is_some()
    }
}

impl From<Option<token::Mut>> for RefFlag {
    fn from(m: Option<token::Mut>) -> Self {
        RefFlag(m)
    }
}

pub(crate) enum ParameterType<'a> {
    VMContextMutPtr,
    SelfRef(RefFlag),
    Context(Option<RefFlag>),
    Ptr(&'a Type),
    Simple(&'a Type, Option<RefFlag>),
}

pub(crate) enum Return<'a> {
    Ptr(&'a Type),
    Simple(&'a Type),
}

pub(crate) struct Parameter<'a> {
    pub(crate) id: Option<&'a Ident>,
    pub(crate) ty: ParameterType<'a>,
}

pub(crate) struct MethodSignature<'a> {
    pub(crate) params: Vec<Parameter<'a>>,
    pub(crate) result: Option<Return<'a>>,
}

pub(crate) fn read_signature<'a>(
    sig: &'a Signature,
    context: &Option<Path>,
) -> MethodSignature<'a> {
    let mut params = Vec::new();
    for i in &sig.inputs {
        match i {
            FnArg::Typed(t) => {
                let id = Some(if let Pat::Ident(ref id) = *t.pat {
                    assert!(id.attrs.is_empty());
                    assert!(id.subpat.is_none());
                    &id.ident
                } else {
                    panic!("no id");
                });
                let ty = match *t.ty {
                    Type::Ptr(ref pt) => match *pt.elem {
                        Type::Path(ref p)
                            if p.path.is_ident("VMContext") && pt.mutability.is_some() =>
                        {
                            ParameterType::VMContextMutPtr
                        }
                        _ => ParameterType::Ptr(&t.ty),
                    },
                    Type::Path(ref tp) => {
                        if context.as_ref().map(|c| *c == tp.path) == Some(true) {
                            ParameterType::Context(None)
                        } else {
                            ParameterType::Simple(&t.ty, None)
                        }
                    }
                    Type::Reference(ref tr) => {
                        let is_context = if let Type::Path(ref tp) = *tr.elem {
                            context.as_ref().map(|c| *c == tp.path) == Some(true)
                        } else {
                            false
                        };
                        if is_context {
                            ParameterType::Context(Some(tr.mutability.clone().into()))
                        } else {
                            ParameterType::Simple(&t.ty, Some(tr.mutability.clone().into()))
                        }
                    }
                    _ => panic!("Unsupported param type declaration"),
                };
                params.push(Parameter { id, ty });
            }
            FnArg::Receiver(r) => {
                assert!(r.attrs.is_empty());
                assert!(r.reference.is_some(), "self needs reference");
                params.push(Parameter {
                    id: None,
                    ty: ParameterType::SelfRef(r.mutability.clone().into()),
                });
            }
        }
    }
    let result = if let ReturnType::Type(_, ref rt) = sig.output {
        Some(match **rt {
            Type::Ptr(_) => Return::Ptr(&**rt),
            Type::Path(_) => Return::Simple(&**rt),
            _ => panic!("Unsupported result type declaration"),
        })
    } else {
        None
    };
    MethodSignature { params, result }
}
