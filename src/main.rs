#[macro_use]
extern crate wasmtime_bindings_macro;

use wasmtime_bindings_common::*;

pub struct MyResult(u32);

impl AbiPrimitive for MyResult {
    type Abi = i32;
    fn create_from_abi(i: i32) -> MyResult {
        MyResult(i as u32)
    }
    fn convert_to_abi(self) -> i32 {
        self.0 as i32
    }
}

pub struct WasiCtx;

impl WasiCtx {
    fn from_vmctx(vmctx: *mut VMContext) -> Self {
        panic!();
    }
}

impl WasmMem for WasiCtx {
    type Abi = i32;

    fn as_ptr<T>(&self, off: i32) -> *mut T {
        panic!();
    }
    fn as_off<T>(&self, off: *mut T) -> i32 {
        panic!();
    }
}

#[wasmtime_trait(module(xmodule), context(WasiCtx))]
trait Module {
    //fn set_vmctx(&mut self, ctx: *mut VMContext) {}
    fn test(&self, wasi: WasiCtx, s: *mut u8, t: u8) -> MyResult;
    fn test2(&self) -> *mut u8;
}

#[wasmtime_method(module(test_mod), context(WasiCtx))]
pub fn test(ctx: *mut VMContext, wasi: WasiCtx, s: *mut u8, t: u8) -> MyResult {
    panic!("test method")
}

#[wasmtime_method(module(test2_mod))]
pub fn test2() -> u32 {
    0
}
/*
mod ttt {
    struct Wrapper {
        instance: InstanceHandle,
    }
    impl Wrapper {
        pub fn new(mut instance: InstanceHandle) -> Wrapper {
            let test = instance.lookup("test").unwrap();
            let test2 = instance.lookup("test2").unwrap();
            Wrapper {
                instance,
            }
        }
    }
    impl Module for Wrapper {
        fn test(&self, wasi: WasiCtx, s: *mut u8, t: u8) -> MyResult {

        }
        fn test2(&self) -> *mut u8 {

        }
    }
}
*/
struct F;
//#[wasm_vmctx_impl]
impl Module for F {
    fn test(&self, wasi: WasiCtx, s: *mut u8, t: u8) -> MyResult {
        panic!()
    }
    fn test2(&self) -> *mut u8 {
        panic!()
    }
}

fn main() {
    /*
    let f = wrap_instance!(instance as Module);
    f.test2();
    let f = wrap_method!(export in instance, |ctx: *mut VMContext, wasi: WasiCtx, s: *mut u8, t: u8| -> MyResult);
    f()*/
}
