use std::ffi::CStr;
use cpp::cpp;
use qmetaobject::prelude::*;

cpp! {{
    #include "QuickFlux"
}}

pub fn register_qml_types() {
  unsafe {
    cpp!([]{ registerQuickFluxQmlTypes(); });
  }
}
