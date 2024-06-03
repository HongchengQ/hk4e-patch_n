use libloading::{Error, Library, Symbol};
use std::arch::asm;
use std::env;
use std::ffi::{CStr, CString};

pub struct VersionDllProxy {
    library: Library,
}

impl VersionDllProxy {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let system_directory = env::var("windir")? + ("\\System32\\");
        let dll_path = system_directory + "version.dll";
        let library = unsafe { Library::new(dll_path) }?;
        Ok(Self { library })
    }

    fn get_function<T>(&self, func_name: &CStr) -> Result<Symbol<T>, Error> {
        unsafe { self.library.get(func_name.to_bytes_with_nul()) }
    }

    pub fn load_functions(&self) -> Result<(), Error> {
        for (i, &name) in FUNCTION_NAMES.iter().enumerate() {
            let fn_ptr = self.get_function::<Symbol<*mut usize>>(name)?;
            unsafe { ORIGINAL_FUNCTIONS[i] = **fn_ptr };
            println!("Loaded function {}@{:p}", name.to_str().unwrap(), unsafe {
                ORIGINAL_FUNCTIONS[i]
            });
        }
        Ok(())
    }
}

macro_rules! count_exprs {
    () => {0usize};
    ($head:expr, $($tail:expr,)*) => {1usize + count_exprs!($($tail,)*)};
}

macro_rules! version_dll_proxy {
    ($($fn_name:ident),*) => {
        static FUNCTION_NAMES: &[&CStr] = &[
            $(
                unsafe { CStr::from_bytes_with_nul_unchecked(concat!(stringify!($fn_name), "\0").as_bytes()) }
            ),*, 
        ];

        #[no_mangle]
        static mut ORIGINAL_FUNCTIONS: [*const usize; count_exprs!($($fn_name,)*)] = [0 as *const usize; count_exprs!($($fn_name,)*)];

        $(
            #[no_mangle]
            extern "C" fn $fn_name() {
                let function_name = FUNCTION_NAMES
                    .iter()
                    .position(|&name| name == CString::new(stringify!($fn_name)).unwrap().as_ref())
                    .unwrap();
                let fn_addr = unsafe { ORIGINAL_FUNCTIONS[function_name] };
                unsafe {
                    asm! {
                        "call {tmp}",
                        tmp = in(reg) fn_addr,
                        clobber_abi("C")
                    }
                }
            }
        )*
    };
}

version_dll_proxy! {
    GetFileVersionInfoA,
    GetFileVersionInfoByHandle,
    GetFileVersionInfoExA,
    GetFileVersionInfoExW,
    GetFileVersionInfoSizeA,
    GetFileVersionInfoSizeExA,
    GetFileVersionInfoSizeExW,
    GetFileVersionInfoSizeW,
    GetFileVersionInfoW,
    VerFindFileA,
    VerFindFileW,
    VerInstallFileA,
    VerInstallFileW,
    VerLanguageNameA,
    VerLanguageNameW,
    VerQueryValueA,
    VerQueryValueW
}
