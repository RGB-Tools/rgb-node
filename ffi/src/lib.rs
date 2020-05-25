use std::os::raw::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::convert::{TryInto, TryFrom};

use serde::Deserialize;

use rgb::lnpbp::bp;
use rgb::lnpbp::rgb::Amount;

use rgb::i9n::*;
use rgb::fungible::{IssueStructure, Outcoins};
use rgb::util::SealSpec;

trait CReturnType: Sized + 'static {
    fn from_opaque(other: &COpaqueStruct) -> Result<&mut Self, String> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        let ty = hasher.finish();

        if other.ty != ty {
            return Err(String::from("Type mismatch"));
        }

        let boxed = unsafe { Box::from_raw(other.ptr.clone() as *mut Self) };
        Ok(Box::leak(boxed))
    }
}
impl CReturnType for Runtime {}
impl CReturnType for String {}
impl CReturnType for () {}

#[repr(C)]
pub struct COpaqueStruct {
    ptr: *const c_void,
    ty: u64,
}

impl COpaqueStruct {
    fn new<T: 'static>(other: T) -> Self {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        let ty = hasher.finish();

        COpaqueStruct {
            ptr: Box::into_raw(Box::new(other)) as *const c_void,
            ty,
        }
    }

    fn raw<T>(ptr: *const T) -> Self {
        COpaqueStruct {
            ptr: ptr as *const c_void,
            ty: 0,
        }
    }
}

#[repr(C)]
pub struct CErrorDetails {
    message: *const c_char,
}

fn string_to_ptr(other: String) -> *const c_char {
    let cstr = match CString::new(other) {
        Ok(cstr) => cstr,
        Err(_) => CString::new(String::from("Error converting string: contains a null-char")).unwrap(),
    };

    cstr.into_raw()
}

fn ptr_to_string(ptr: *mut c_char) -> Result<String, String> {
    unsafe {
        CStr::from_ptr(ptr).to_str().map(|s| s.into()).map_err(|e| format!("{:?}", e))
    }
}


#[repr(C)]
pub enum CResultValue {
    Ok,
    Err,
}

#[repr(C)]
pub struct CResult {
    result: CResultValue,
    inner: COpaqueStruct,
}

impl<T: 'static, E> From<Result<T, E>> for CResult
where
    E: std::fmt::Debug,
{
	fn from(other: Result<T, E>) -> Self {
        match other {
            Ok(d) => CResult { result: CResultValue::Ok, inner: COpaqueStruct::new(d) },
            Err(e) => CResult { result: CResultValue::Err, inner: COpaqueStruct::raw(string_to_ptr(format!("{:?}", e))) },
        }
	}
}

#[no_mangle]
pub extern "C" fn start_rgb() -> CResult {
    println!("Starting RGB...");

    Runtime::init(Config::default()).into()
}

#[derive(Debug, Deserialize)]
struct IssueArgs {
    #[serde(with = "serde_with::rust::display_fromstr")]
    network: bp::Network,
    ticker: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    issue_structure: IssueStructure,
    #[serde(default)]
    allocations: Vec<Outcoins>,
    precision: u8,
    #[serde(default)]
    prune_seals: Vec<SealSpec>,
    #[serde(default)]
    dust_limit: Option<Amount>,
}

fn _issue(runtime: &COpaqueStruct, json: *mut c_char) -> Result<(), String> {
    let runtime = Runtime::from_opaque(runtime)?;
    let data: IssueArgs = serde_json::from_str(ptr_to_string(json)?.as_str()).map_err(|e| format!("{:?}", e))?;
    println!("{:?}", data);

    runtime.issue(
        data.network,
        data.ticker,
        data.name,
        data.description,
        data.issue_structure,
        data.allocations,
        data.precision,
        data.prune_seals,
        data.dust_limit,
    ).map_err(|e| format!("{:?}", e))
}

#[no_mangle]
pub extern "C" fn issue(runtime: &COpaqueStruct, json: *mut c_char) -> CResult {
    _issue(runtime, json).into()
}