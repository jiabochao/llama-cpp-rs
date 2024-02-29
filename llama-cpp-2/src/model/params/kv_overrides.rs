//! Key-value overrides for a model.

use crate::model::params::LlamaModelParams;
use std::ffi::{CStr, CString};

#[derive(Default, Clone)]
pub struct KvOverrideStorage {
    inner: Vec<llama_cpp_sys_2::llama_model_kv_override>,
}

impl KvOverrideStorage {
    fn new() -> KvOverrideStorage {
        Default::default()
    }
}

/// An override value for a model parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamOverrideValue {
    /// A string value
    Bool(bool),
    /// A float value
    Float(f64),
    /// A integer value
    Int(i64),
}

impl ParamOverrideValue {
    pub(crate) fn tag(&self) -> llama_cpp_sys_2::llama_model_kv_override_type {
        match self {
            ParamOverrideValue::Bool(_) => llama_cpp_sys_2::LLAMA_KV_OVERRIDE_TYPE_BOOL,
            ParamOverrideValue::Float(_) => llama_cpp_sys_2::LLAMA_KV_OVERRIDE_TYPE_FLOAT,
            ParamOverrideValue::Int(_) => llama_cpp_sys_2::LLAMA_KV_OVERRIDE_TYPE_INT,
        }
    }

    pub(crate) fn value(&self) -> llama_cpp_sys_2::llama_model_kv_override__bindgen_ty_1 {
        match self {
            ParamOverrideValue::Bool(value) => {
                llama_cpp_sys_2::llama_model_kv_override__bindgen_ty_1 { bool_value: *value }
            }
            ParamOverrideValue::Float(value) => {
                llama_cpp_sys_2::llama_model_kv_override__bindgen_ty_1 {
                    float_value: *value,
                }
            }
            ParamOverrideValue::Int(value) => {
                llama_cpp_sys_2::llama_model_kv_override__bindgen_ty_1 { int_value: *value }
            }
        }
    }
}

impl From<&llama_cpp_sys_2::llama_model_kv_override> for ParamOverrideValue {
    fn from(
        llama_cpp_sys_2::llama_model_kv_override {
            key: _,
            tag,
            __bindgen_anon_1,
        }: &llama_cpp_sys_2::llama_model_kv_override,
    ) -> Self {
        match *tag {
            llama_cpp_sys_2::LLAMA_KV_OVERRIDE_TYPE_INT => {
                ParamOverrideValue::Int(unsafe { __bindgen_anon_1.int_value })
            }
            llama_cpp_sys_2::LLAMA_KV_OVERRIDE_TYPE_FLOAT => {
                ParamOverrideValue::Float(unsafe { __bindgen_anon_1.float_value })
            }
            llama_cpp_sys_2::LLAMA_KV_OVERRIDE_TYPE_BOOL => {
                ParamOverrideValue::Bool(unsafe { __bindgen_anon_1.bool_value })
            }
            _ => unreachable!("Unknown tag of {tag}"),
        }
    }
}

/// A struct implementing [`IntoIterator`] over the key-value overrides for a model.
pub struct KvOverrides<'a> {
    model_params: &'a LlamaModelParams<Vec<llama_cpp_sys_2::llama_model_kv_override>>,
}

impl KvOverrides<'_> {
    pub(super) fn new(
        model_params: &LlamaModelParams<Vec<llama_cpp_sys_2::llama_model_kv_override>>,
    ) -> KvOverrides {
        KvOverrides { model_params }
    }
}

impl<'a> IntoIterator for KvOverrides<'a> {
    // I'm fairly certain this could be written returning by reference, but I'm not sure how to do it safely. I do not
    // expect this to be a performance bottleneck so the copy should be fine. (let me know if it's not fine!)
    type Item = (CString, ParamOverrideValue);
    type IntoIter = KvOverrideValueIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        KvOverrideValueIterator {
            model_params: self.model_params,
            current: 0,
        }
    }
}

/// An iterator over the key-value overrides for a model.
#[derive(Debug)]
pub struct KvOverrideValueIterator<'a> {
    model_params: &'a LlamaModelParams<Vec<llama_cpp_sys_2::llama_model_kv_override>>,
    current: usize,
}

impl<'a> Iterator for KvOverrideValueIterator<'a> {
    type Item = (CString, ParamOverrideValue);

    fn next(&mut self) -> Option<Self::Item> {
        let overrides = self.model_params.params.kv_overrides;
        if overrides.is_null() {
            return None;
        }

        // SAFETY: llama.cpp seems to guarantee that the last element contains an empty key or is valid. We've checked
        // the prev one in the last iteration, the next one should be valid or 0 (and thus safe to deref)
        let current = unsafe { *overrides.add(self.current) };

        eprintln!("reading from {:?}", unsafe { overrides.add(self.current) });

        if current.key[0] == 0 {
            eprintln!("contains no data");
            return None;
        }

        eprintln!("contains data: {:?}", current.key);

        let value = ParamOverrideValue::from(&current);

        let key = unsafe { CStr::from_ptr(current.key.as_ptr()).to_owned() };

        eprintln!("key from CStr: {:?}", key);

        self.current += 1;
        Some((key, value))
    }
}
