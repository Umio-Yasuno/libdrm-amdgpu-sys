use crate::AMDGPU::DEVICE_HANDLE;
use crate::*;

pub use bindings::{
    amdgpu_device_handle,
    // amdgpu_device_initialize,
    amdgpu_gds_resource_info,
    amdgpu_gpu_info,
    // amdgpu_heap_info,
    drm_amdgpu_info_device,
    drm_amdgpu_info_gds,
    drm_amdgpu_memory_info,
    drm_amdgpu_info_vce_clock_table,
};
use bindings::{
    AMDGPU_INFO_NUM_BYTES_MOVED,
    AMDGPU_INFO_DEV_INFO,
    AMDGPU_INFO_GDS_CONFIG,
    AMDGPU_INFO_MEMORY,
    AMDGPU_INFO_VRAM_USAGE,
    AMDGPU_INFO_VCE_CLOCK_TABLE,
    AMDGPU_INFO_NUM_VRAM_CPU_PAGE_FAULTS,
};
use core::mem::{size_of, MaybeUninit};

pub struct DeviceHandle(pub(crate) DEVICE_HANDLE);

unsafe impl Send for DeviceHandle {}
unsafe impl Sync for DeviceHandle {}

impl DeviceHandle {
    pub fn init(fd: i32) -> Result<(Self, u32, u32), i32> {
        unsafe {
            let mut amdgpu_dev: MaybeUninit<amdgpu_device_handle> = MaybeUninit::uninit();
            let mut major: MaybeUninit<u32> = MaybeUninit::zeroed();
            let mut minor: MaybeUninit<u32> = MaybeUninit::zeroed();

            let r = bindings::amdgpu_device_initialize(
                fd,
                major.as_mut_ptr(),
                minor.as_mut_ptr(),
                amdgpu_dev.as_mut_ptr(),
            );

            let [major, minor] = [major.assume_init(), minor.assume_init()];
            let amdgpu_dev = Self(amdgpu_dev.assume_init());

            query_error!(r);

            return Ok((amdgpu_dev, major, minor));
        }
    }

    fn deinit(&self) -> Result<i32, i32> {
        let r = unsafe { bindings::amdgpu_device_deinitialize(self.0) };

        query_error!(r);

        Ok(r)
    }

    pub fn get_fd(&self) -> i32 {
        unsafe { bindings::amdgpu_device_get_fd(self.0) }
    }

    pub fn get_drm_version(&self) -> Result<(i32, i32, i32), ()> {
        let fd = self.get_fd();
        let mut _drm_ver_ptr = unsafe { bindings::drmGetVersion(fd) };

        if _drm_ver_ptr.is_null() {
            return Err(());
        }

        let ver = unsafe { (
            (*_drm_ver_ptr).version_major,
            (*_drm_ver_ptr).version_minor,
            (*_drm_ver_ptr).version_patchlevel,
        ) };

        unsafe { bindings::drmFreeVersion(_drm_ver_ptr) }

        Ok(ver)
    }

    pub fn read_grbm(&self) -> Result<u32, i32> {
        // TODO: check AMDGPU Family,
        // <=GFX8: 0x2004, Vega/GFX9: 0x4, RDNA1/RDNA2: 0xDA4
        self.read_mm_registers(0x2004)
    }

    pub fn read_grbm2(&self) -> Result<u32, i32> {
        // TODO: check AMDGPU Family,
        // <=GFX8: 0x2002, Vega/GFX9: 0x2, RDNA1/RDNA2: 0xDA2
        self.read_mm_registers(0x2002)
    }

    pub fn read_mm_registers(&self, offset: u32) -> Result<u32, i32> {
        unsafe {
            let mut out: MaybeUninit<u32> = MaybeUninit::zeroed();

            let r = bindings::amdgpu_read_mm_registers(
                self.0,
                offset, // DWORD offset
                1, // count
                0xFFFF_FFFF, // instance mask, full mask
                0, // flags
                out.as_mut_ptr(),
            );

            let out = out.assume_init();

            query_error!(r);

            Ok(out)
        }
    }

    #[cfg(feature = "std")]
    pub fn get_marketing_name(&self) -> Result<String, std::str::Utf8Error> {
        use core::ffi::CStr;

        let c_str = unsafe {
            let mark_name = bindings::amdgpu_get_marketing_name(self.0);

            if mark_name.is_null() {
                eprintln!("libdrm_amdgpu_sys: ASIC not found in amdgpu.ids");
                return Ok("".to_string());
            }

            CStr::from_ptr(mark_name)
        };

        Ok(c_str.to_str()?.to_string())
    }

    pub fn query_gpu_info(&self) -> Result<amdgpu_gpu_info, i32> {
        unsafe {
            let mut gpu_info: MaybeUninit<amdgpu_gpu_info> = MaybeUninit::uninit();

            let r = bindings::amdgpu_query_gpu_info(self.0, gpu_info.as_mut_ptr());

            let gpu_info = gpu_info.assume_init();

            query_error!(r);

            return Ok(gpu_info);
        }
    }

    pub fn query_gds_info(&self) -> Result<amdgpu_gds_resource_info, i32> {
        unsafe {
            let mut gds_info: MaybeUninit<amdgpu_gds_resource_info> = MaybeUninit::uninit();

            let r = bindings::amdgpu_query_gds_info(self.0, gds_info.as_mut_ptr());

            let gds_info = gds_info.assume_init();

            query_error!(r);

            return Ok(gds_info);
        }
    }

    pub fn query_sw_info(&self, info: amdgpu_sw_info) -> Result<u32, i32> {
        unsafe {
            let mut val: MaybeUninit<u32> = MaybeUninit::zeroed();

            let r = bindings::amdgpu_query_sw_info(
                self.0,
                info as u32,
                val.as_mut_ptr() as *mut ::core::ffi::c_void,
            );

            let val = val.assume_init();

            query_error!(r);

            return Ok(val);
        }
    }

    fn query<T>(&self, info_id: ::core::ffi::c_uint) -> Result<T, i32> {
        unsafe {
            let mut dev: MaybeUninit<T> = MaybeUninit::uninit();

            let r = bindings::amdgpu_query_info(
                self.0,
                info_id,
                size_of::<T>() as u32,
                dev.as_mut_ptr() as *mut ::core::ffi::c_void,
            );

            let dev = dev.assume_init();

            query_error!(r);

            return Ok(dev);
        }
    }

    pub fn device_info(&self) -> Result<drm_amdgpu_info_device, i32> {
        Self::query(self, AMDGPU_INFO_DEV_INFO)
    }

    pub fn memory_info(&self) -> Result<drm_amdgpu_memory_info, i32> {
        Self::query(self, AMDGPU_INFO_MEMORY)
    }

    pub fn vram_usage_info(&self) -> Result<u64, i32> {
        Self::query(self, AMDGPU_INFO_VRAM_USAGE)
    }

    pub fn gds_info(&self) -> Result<drm_amdgpu_info_gds, i32> {
        Self::query(self, AMDGPU_INFO_GDS_CONFIG)
    }

    /// drm_amdgpu_info_vce_clock_table is invalid
    pub fn vce_clock_info(&self) -> Result<drm_amdgpu_info_vce_clock_table, i32> {
        Self::query(self, AMDGPU_INFO_VCE_CLOCK_TABLE)
    }

    pub fn num_vram_cpu_page_failts(&self) -> Result<u64, i32> {
        Self::query(self, AMDGPU_INFO_NUM_VRAM_CPU_PAGE_FAULTS)
    }

    pub fn num_bytes_moved(&self) -> Result<u64, i32> {
        Self::query(self, AMDGPU_INFO_NUM_BYTES_MOVED)
    }

    pub fn get_pci_bus_info(&self) -> Result<PCI::BUS_INFO, i32> {
        PCI::BUS_INFO::drm_get_device2(self.get_fd())
    }

    const PCI_PATH: &str = "/sys/bus/pci/devices";

    #[cfg(feature = "std")]
    fn get_first_line(path: &String) -> Result<String, std::io::Error> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let f = File::open(path)?;
        let mut first_line = String::new();
        let mut buf = BufReader::new(f);
        buf.read_line(&mut first_line)?;

        Ok(first_line)
    }

    #[cfg(feature = "std")]
    fn trim_dpm_clk(line: &String) -> Result<u64, std::num::ParseIntError> {
        const MHZ: &str = "Mhz";
        let mut tmp = String::new();

        /* 0: 214Mhz */
        for s in line.split(' ') {
            if s.ends_with(MHZ) {
                tmp = s.trim_end_matches(MHZ).to_string();
            }
        }

        tmp.parse::<u64>()
    }

    #[cfg(feature = "std")]
    fn get_min_clock(&self, file_name: &str) -> Option<u64> {
        if let Ok(pci) = self.get_pci_bus_info() {
            let path = format!("{PCI_PATH}/{pci}/{file_name}", PCI_PATH = Self::PCI_PATH);

            if let Ok(line) = Self::get_first_line(&path) {
                if let Ok(clk) = Self::trim_dpm_clk(&line) {
                    return Some(clk);
                }
            }
        }

        None
    }

    #[cfg(feature = "std")]
    pub fn get_min_gpu_clock_from_sysfs(&self) -> Option<u64> {
        Self::get_min_clock(self, "pp_dpm_sclk")
    }

    #[cfg(feature = "std")]
    pub fn get_min_memory_clock_from_sysfs(&self) -> Option<u64> {
        Self::get_min_clock(self, "pp_dpm_mclk")
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        self.deinit().unwrap();
    }
}

#[repr(u32)]
pub enum amdgpu_sw_info {
    address32_hi = 0,
}
