//! Extension traits
use wasmer::WasmPtr;

pub trait IsNull {
    fn is_null(&self) -> bool;
}

impl<T: Copy, Ty> IsNull for WasmPtr<T, Ty> {
    #[inline(always)]
    fn is_null(&self) -> bool {
        self.offset() == 0
    }
}
