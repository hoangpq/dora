use std::collections::hash_map::HashMap;
use std::mem::size_of;

use baseline::fct::JitFct;
use cpu::{Mem, REG_FP, REG_PARAMS, REG_RESULT, REG_SP};
use ctxt::{Context, get_ctxt};
use masm::MacroAssembler;
use mem;
use stacktrace::DoraToNativeInfo;
use ty::{BuiltinType, MachineMode};

pub struct NativeFcts {
    map: HashMap<*const u8, JitFct>,
}

impl NativeFcts {
    pub fn new() -> NativeFcts {
        NativeFcts { map: HashMap::new() }
    }

    pub fn find_fct(&self, ptr: *const u8) -> Option<*const u8> {
        self.map.get(&ptr).map(|jit_fct| jit_fct.fct_start as *const u8)
    }

    pub fn insert_fct(&mut self, ptr: *const u8, fct: JitFct) -> *const u8 {
        self.map.entry(ptr).or_insert(fct).fct_start as *const u8
    }
}

pub struct InternalFct<'a> {
    pub ptr: *const u8,
    pub args: &'a [BuiltinType],
    pub return_type: BuiltinType,
}

pub fn generate<'a, 'ast: 'a>(ctxt: &'a Context<'ast>,
                              fct: InternalFct)
                              -> JitFct {
    let ngen = NativeGen {
        ctxt: ctxt,
        masm: MacroAssembler::new(),
        fct: fct,
    };

    ngen.generate()
}

struct NativeGen<'a, 'ast: 'a> {
    ctxt: &'a Context<'ast>,
    masm: MacroAssembler,

    fct: InternalFct<'a>,
}

impl<'a, 'ast> NativeGen<'a, 'ast>
    where 'ast: 'a
{
    pub fn generate(mut self) -> JitFct {
        let save_return = self.fct.return_type != BuiltinType::Unit;
        let args = self.fct.args.len();

        let framesize = size_of::<DoraToNativeInfo>() as i32 + if save_return { 8 } else { 0 } +
                        (args * 8) as i32;

        let framesize = mem::align_i32(framesize, 16);

        let offset_return = 0;
        let offset_args = offset_return + if save_return { 8 } else { 0 };
        // let offset_sfi = offset_args + self.args * 8;

        self.masm.prolog(framesize);

        assert!(args <= REG_PARAMS.len());

        for (ind, &reg) in REG_PARAMS.iter().take(args as usize).enumerate() {
            self.masm.store_mem(MachineMode::Ptr,
                                Mem::Base(REG_SP, offset_args + ind as i32 * 8),
                                reg.into());
        }

        self.masm.copy_reg(MachineMode::Ptr, REG_PARAMS[0], REG_FP);
        self.masm.direct_call_without_info(start_native_call as *const u8);

        for (ind, &reg) in REG_PARAMS.iter().take(args as usize).enumerate() {
            self.masm.load_mem(MachineMode::Ptr,
                               reg.into(),
                               Mem::Base(REG_SP, offset_args + ind as i32 * 8));
        }

        self.masm.direct_call_without_info(self.fct.ptr);

        if save_return {
            self.masm.store_mem(MachineMode::Ptr, Mem::Base(REG_SP, 0), REG_RESULT.into());
        }

        self.masm.direct_call_without_info(finish_native_call as *const u8);

        if save_return {
            self.masm.load_mem(MachineMode::Ptr, REG_RESULT.into(), Mem::Base(REG_SP, 0));
        }

        self.masm.epilog(framesize);

        self.masm.jit(self.ctxt, framesize)
    }
}

fn start_native_call(fp: *const u8) {
    unsafe {
        // fp is framepointer of native stub

        // get framepointer of dora function and return address into dora
        let dora_ra = *(fp.offset(8) as *const usize);
        let dora_fp = *(fp as *const usize);

        let sfi_size = size_of::<DoraToNativeInfo>() as isize;
        let sfi: *mut DoraToNativeInfo = fp.offset(-sfi_size) as *mut DoraToNativeInfo;
        let sfi: &mut DoraToNativeInfo = &mut *sfi;

        sfi.sp = 0;
        sfi.fp = dora_fp;
        sfi.ra = dora_ra;
        sfi.xpc = sfi.ra - 1;

        let ctxt = get_ctxt();
        ctxt.push_sfi(sfi);
    }
}

fn finish_native_call() {
    get_ctxt().pop_sfi();
}
