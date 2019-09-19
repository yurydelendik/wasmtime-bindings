pub use cranelift_codegen as codegen;
pub use wasmtime_runtime::VMContext;

pub trait IntoIRType {
    fn into_ir_type() -> codegen::ir::Type;
}

pub trait AbiRet {
    type Abi: IntoIRType;
    fn convert_to_abi(self) -> Self::Abi;
    fn create_from_abi(ret: Self::Abi) -> Self;
}

pub trait AbiParam {
    type Abi: IntoIRType;
    fn convert_to_abi(self) -> Self::Abi;
    fn create_from_abi(arg: Self::Abi) -> Self;
}

macro_rules! cast32 {
    ($($i:ident)*) => ($(
        impl AbiRet for $i {
            type Abi = i32;

            fn convert_to_abi(self) -> Self::Abi {
                self as i32
            }

            fn create_from_abi(ret: Self::Abi) -> Self {
                ret as $i
            }
        }

        impl AbiParam for $i {
            type Abi = i32;

            fn convert_to_abi(self) -> Self::Abi {
                self as i32
            }

            fn create_from_abi(arg: Self::Abi) -> Self {
                arg as $i
            }
        }
    )*)
}

macro_rules! cast64 {
    ($($i:ident)*) => ($(
        impl AbiRet for $i {
            type Abi = i64;

            fn convert_to_abi(self) -> Self::Abi {
                self as i64
            }

            fn create_from_abi(ret: Self::Abi) -> Self {
                ret as $i
            }
        }

        impl AbiParam for $i {
            type Abi = i64;

            fn convert_to_abi(self) -> Self::Abi {
                self as i64
            }

            fn create_from_abi(arg: Self::Abi) -> Self {
                arg as $i
            }
        }
    )*)
}

cast32!(i8 i16 i32 u8 u16 u32);
cast64!(i64 u64);

pub trait WasmMem {
    type Abi;
    fn as_ptr<T>(&self, off: Self::Abi) -> *mut T;
    fn as_off<T>(&self, ptr: *mut T) -> Self::Abi;
}

pub struct VMContextWrapper(*mut VMContext);

impl WasmMem for VMContextWrapper {
    type Abi = i32;
    fn as_ptr<T>(&self, _off: Self::Abi) -> *mut T {
        unimplemented!();
    }
    fn as_off<T>(&self, _ptr: *mut T) -> Self::Abi {
        unimplemented!();
    }
}

impl IntoIRType for i32 {
    fn into_ir_type() -> codegen::ir::Type {
        codegen::ir::types::I32
    }
}

impl IntoIRType for i64 {
    fn into_ir_type() -> codegen::ir::Type {
        codegen::ir::types::I64
    }
}

pub fn get_ir_type<T: IntoIRType>() -> codegen::ir::Type {
    T::into_ir_type()
}
