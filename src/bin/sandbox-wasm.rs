#[cfg(target_os = "emscripten")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_os = "emscripten", wasm_bindgen)]
pub struct Sandbox {
    spec: String,
}

#[cfg_attr(target_os = "emscripten", wasm_bindgen)]
impl Sandbox {
    #[cfg_attr(target_os = "emscripten", wasm_bindgen(constructor))]
    pub fn new(spec: String) -> Sandbox {
        Sandbox { spec }
    }

    pub fn verify(&self) -> String {
        verifier::verify(self.spec.as_str()).message.to_string()
    }
}

fn main() {}
