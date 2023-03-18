use libdrm_amdgpu_sys::*;

fn main() {
    let (amdgpu_dev, _major, _minor) = {
        use std::fs::File;
        use std::os::fd::IntoRawFd;

        let fd = File::open("/dev/dri/renderD128").unwrap();

        AMDGPU::DeviceHandle::init(fd.into_raw_fd()).unwrap()
    };

    if let Ok(drm_ver) = amdgpu_dev.get_drm_version() {
        let (major, minor, patchlevel) = drm_ver;
        println!("drm version: {major}.{minor}.{patchlevel}");
    }

    if let Ok(mark_name) = amdgpu_dev.get_marketing_name() {
        println!("Marketing Name: [{mark_name}]");
    }

    if let Ok(ext_info) = amdgpu_dev.device_info() {
        use AMDGPU::GPU_INFO;

        // println!("\n{ext_info:#X?}\n");
        let gpu_type = if ext_info.is_apu() {
            "APU"
        } else {
            "dGPU"
        };

        println!(
            "DeviceID.RevID: {:#0X}.{:#0X}",
            ext_info.device_id(),
            ext_info.pci_rev_id()
        );

        println!();
        println!("Family:\t\t{}", ext_info.get_family_name());
        println!("ASIC Name:\t{}", ext_info.get_asic_name());
        println!("Chip class:\t{}", ext_info.get_chip_class());
        println!("GPU Type:\t{gpu_type}");

        let max_good_cu_per_sa = ext_info.get_max_good_cu_per_sa();
        let min_good_cu_per_sa = ext_info.get_min_good_cu_per_sa();

        println!();
        println!("Shader Engine (SE):\t\t{:3}", ext_info.max_se());
        println!("Shader Array (SA/SH) per SE:\t{:3}", ext_info.max_sa_per_se());
        if max_good_cu_per_sa != min_good_cu_per_sa {
            println!("CU per SA[0]:\t\t\t{:3}", max_good_cu_per_sa);
            println!("CU per SA[1]:\t\t\t{:3}", min_good_cu_per_sa);
        } else {
            println!("CU per SA:\t\t\t{:3}", max_good_cu_per_sa);
        }
        println!("Total Compute Unit:\t\t{:3}", ext_info.cu_active_number());

        if let Ok(pci_bus) = amdgpu_dev.get_pci_bus_info() {
            if let Some(min_clk) = amdgpu_dev.get_min_gpu_clock_from_sysfs(&pci_bus) {
                println!("Min Engine Clock:\t{min_clk:4} MHz");
            }
        }

        // KHz / 1000
        println!("Max Engine Clock:\t{:4} MHz", ext_info.max_engine_clock() / 1000);
        println!("Peak FP32:\t\t{} GFLOPS", ext_info.peak_gflops());

        println!();
        println!("VRAM Type:\t\t{}", ext_info.get_vram_type());
        println!("VRAM Bit Width:\t\t{}-bit", ext_info.vram_bit_width);
        if let Ok(pci_bus) = amdgpu_dev.get_pci_bus_info() {
            if let Some(min_clk) = amdgpu_dev.get_min_memory_clock_from_sysfs(&pci_bus) {
                println!("Min Memory Clock:\t{min_clk:4} MHz");
            }
        }
        println!("Max Memory Clock:\t{:4} MHz", ext_info.max_memory_clock() / 1000);
        println!("Peak Memory BW:\t\t{} GB/s", ext_info.peak_memory_bw_gb());
        println!(
            "L2cache:\t\t{} KiB ({} Banks)",
            ext_info.calc_l2_cache_size() / 1024,
            ext_info.num_tcc_blocks
        );
    }

    if let Ok(info) = amdgpu_dev.memory_info() {
        println!();
        println!(
            "VRAM Usage:\t\t\t{usage}/{total} MiB",
            usage = info.vram.heap_usage / 1024 / 1024,
            total = info.vram.total_heap_size / 1024 / 1024,
        );
        println!(
            "CPU Accessible VRAM Usage:\t{usage}/{total} MiB",
            usage = info.cpu_accessible_vram.heap_usage / 1024 / 1024,
            total = info.cpu_accessible_vram.total_heap_size / 1024 / 1024,
        );
        println!(
            "GTT Usage:\t\t\t{usage}/{total} MiB",
            usage = info.gtt.heap_usage / 1024 / 1024,
            total = info.gtt.total_heap_size / 1024 / 1024,
        );
    }

    {
        use AMDGPU::HW_IP::*;

        let ip_list = [
            HW_IP_TYPE::GFX,
            HW_IP_TYPE::COMPUTE,
            HW_IP_TYPE::DMA,
            HW_IP_TYPE::UVD,
            HW_IP_TYPE::VCE,
            HW_IP_TYPE::UVD_ENC,
            HW_IP_TYPE::VCN_DEC,
            HW_IP_TYPE::VCN_ENC,
            HW_IP_TYPE::VCN_JPEG,
        ];

        println!("\nHardware IP info:");

        for ip_type in &ip_list {
            if let (Ok(ip_info), Ok(ip_count)) = (
                amdgpu_dev.query_hw_ip_info(*ip_type, 0),
                amdgpu_dev.query_hw_ip_count(*ip_type),
            ) {
                let (major, minor) = ip_info.version();
                let queues = ip_info.num_queues();

                if queues == 0 {
                    continue;
                }

                println!(
                    "{ip_type:8} count: {ip_count}, ver: {major:2}.{minor}, queues: {queues}",
                    ip_type = ip_type.to_string(),
                );
            }
        }
    }

    {
        use AMDGPU::FW_VERSION::*;

        let fw_list = [
            FW_TYPE::VCE,
            FW_TYPE::UVD,
            FW_TYPE::GMC,
            FW_TYPE::GFX_ME,
            FW_TYPE::GFX_PFP,
            FW_TYPE::GFX_CE,
            FW_TYPE::GFX_RLC,
            FW_TYPE::GFX_MEC,
            FW_TYPE::SMC,
            FW_TYPE::SDMA,
            FW_TYPE::SOS,
            FW_TYPE::ASD,
            FW_TYPE::VCN,
            FW_TYPE::GFX_RLC_RESTORE_LIST_CNTL,
            FW_TYPE::GFX_RLC_RESTORE_LIST_GPM_MEM,
            FW_TYPE::GFX_RLC_RESTORE_LIST_SRM_MEM,
            FW_TYPE::DMCU,
            FW_TYPE::TA,
            FW_TYPE::DMCUB,
            FW_TYPE::TOC,
        ];

        println!("\nFirmware info:");

        for fw_type in &fw_list {
            let fw_info = match amdgpu_dev.query_firmware_version(*fw_type, 0, 0) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let (ver, ftr) = (fw_info.version, fw_info.feature);

            if ver == 0 {
                continue;
            }

            println!(
                "{fw_type:<8} ver: {ver:>#10X}, feature: {ftr:>3}",
                fw_type = fw_type.to_string(),
            );
        }
    }

    if let [Ok(dec), Ok(enc)] = [
        amdgpu_dev.get_video_caps(AMDGPU::VIDEO_CAPS::CAP_TYPE::DECODE),
        amdgpu_dev.get_video_caps(AMDGPU::VIDEO_CAPS::CAP_TYPE::ENCODE),
    ] {
        use AMDGPU::VIDEO_CAPS::*;

        let codec_list = [
            CODEC::MPEG2,
            CODEC::MPEG4,
            CODEC::VC1,
            CODEC::MPEG4_AVC,
            CODEC::HEVC,
            CODEC::JPEG,
            CODEC::VP9,
            CODEC::AV1,
        ];

        println!("\nVideo caps:");

        for codec in &codec_list {
            let [dec_cap, enc_cap] = [dec, enc].map(|type_| type_.get_codec_info(*codec));

            println!("{codec}:");
            println!("    Decode: w {:>5}, h {:>5}", dec_cap.max_width, dec_cap.max_height);
            println!("    Encode: w {:>5}, h {:>5}", enc_cap.max_width, enc_cap.max_height);
        }
    }

    if let Ok(bus_info) = amdgpu_dev.get_pci_bus_info() {
        let cur = bus_info.get_link_info(PCI::STATUS::Current);
        let max = bus_info.get_link_info(PCI::STATUS::Max);

        println!("\nPCI (domain:bus:dev.func): {bus_info}");
        println!("Current Link: Gen{}x{}", cur.gen, cur.width);
        println!("Max     Link: Gen{}x{}", max.gen, max.width);
    }

    if let Ok(vbios) = amdgpu_dev.get_vbios_info() {
        println!("\nVBIOS info:");
        println!("name: [{}]", vbios.name);
        println!("pn: [{}]", vbios.pn);
        println!("ver: [{}]", vbios.ver);
        println!("date: [{}]", vbios.date);
    }

/*
    if let Ok(vce_clock) = amdgpu_dev.vce_clock_info() {
        println!("\n{vce_clock:#?}");
    }
*/

    {
        use AMDGPU::SENSOR_INFO::*;

        let sensors = [
            SENSOR_TYPE::GFX_SCLK,
            SENSOR_TYPE::GFX_MCLK,
            SENSOR_TYPE::GPU_TEMP,
            SENSOR_TYPE::GPU_LOAD,
            SENSOR_TYPE::GPU_AVG_POWER,
            SENSOR_TYPE::VDDNB,
            SENSOR_TYPE::VDDGFX,
            SENSOR_TYPE::STABLE_PSTATE_GFX_SCLK,
            SENSOR_TYPE::STABLE_PSTATE_GFX_MCLK,
        ];

        println!("\nSensors:");

        for s in &sensors {
            if let Ok(val) = amdgpu_dev.sensor_info(*s) {
                println!("{s:?}: {val}");
            } else {
                println!("{s:?}: not supported");
            }
        }
    }
}
