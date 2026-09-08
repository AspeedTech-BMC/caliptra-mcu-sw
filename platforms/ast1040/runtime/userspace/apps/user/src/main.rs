// Licensed under the Apache-2.0 license

#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]
#![allow(static_mut_refs)]

use caliptra_mcu_libtockasync::TockExecutor;
#[allow(unused)]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[allow(unused)]
use embassy_sync::{lazy_lock::LazyLock, signal::Signal};

// Re-export the unified logging macros from userlog so
// `crate::log_*!(...)` call sites in this crate resolve. Dev builds write
// console text; release builds emit defmt frames.
#[allow(unused_imports)]
pub use caliptra_mcu_userlog::{log_debug, log_error, log_info, log_trace, log_warn};
#[allow(unused_imports)]
pub(crate) use caliptra_mcu_userlog::{Bytes, Dbg, Hex32};

#[allow(unused_imports)]
pub use caliptra_mcu_libsyscall_caliptra::console_writeln;

pub(crate) mod auth_keys;
mod caliptra_cmd_handler;
// SPDM is the only current consumer; widen this gate when another protocol
// integrates the boot-provisioned certificate chains.
#[cfg(feature = "spdm")]
mod cert_store;
#[cfg(any(
    feature = "test-defmt-logging-mailbox",
    feature = "test-defmt-logging-release",
    feature = "test-defmt-logging-vdm"
))]
mod defmt_test;
#[cfg(any(
    feature = "firmware-update",
    feature = "test-firmware-update-streaming"
))]
mod firmware_update;
mod image_loader;
mod mcu_mbox;
mod measurement;
mod soc_image_descriptors {
    include!(concat!(env!("OUT_DIR"), "/soc_image_descriptors.rs"));
}
#[cfg(target_arch = "riscv32")]
mod panic;
#[cfg(feature = "spdm")]
mod spdm;
#[cfg(feature = "mctp-vdm-service")]
mod vdm;

#[cfg(target_arch = "riscv32")]
mod riscv;

fn log_spawn_error() {}

pub static EXECUTOR: LazyLock<TockExecutor> = LazyLock::new(TockExecutor::new);

#[cfg(target_arch = "riscv32")]
#[embassy_executor::task]
async fn start() {
    async_main().await;
}

pub(crate) async fn async_main() {
    // Initialize measurement state before spawning any task that could consume
    // it (image loading, firmware update, SPDM/evidence, MCU mailbox).
    let soc_image_load_list = soc_image_descriptors::SOC_IMAGE_LOAD_LIST;
    measurement::boot_init(
        measurement::attestation_manifest_bytes(),
        soc_image_load_list,
    )
    .await;

    #[cfg(feature = "spdm")]
    match cert_store::boot_init().await {
        Ok(()) => {
            spdm::spawn_spdm_tasks(&EXECUTOR.get().spawner());
        }
        Err(e) => {
            let mut cw = caliptra_mcu_libtock_console::Console::<
                caliptra_mcu_libsyscall_caliptra::DefaultSyscalls,
            >::writer();
            crate::log_error!(
                cw,
                "CERT_STORE: boot initialization failed: 0x{}",
                crate::Hex32(u32::from(e))
            );
        }
    }

    EXECUTOR
        .get()
        .spawner()
        .spawn(image_loader::image_loading_task(soc_image_load_list))
        .map_err(|_| log_spawn_error())
        .ok();

    EXECUTOR
        .get()
        .spawner()
        .spawn(mcu_mbox::mcu_mbox_task())
        .map_err(|_| log_spawn_error())
        .ok();

    #[cfg(feature = "test-mcu-mbox-fips-periodic")]
    EXECUTOR
        .get()
        .spawner()
        .spawn(caliptra_mcu_mbox_lib::fips_periodic::fips_periodic_task())
        .map_err(|_| log_spawn_error())
        .ok();

    #[cfg(feature = "mctp-vdm-service")]
    EXECUTOR
        .get()
        .spawner()
        .spawn(vdm::vdm_task())
        .map_err(|_| log_spawn_error())
        .ok();

    // Production userspace defmt logging: drain staged frames to the flash log
    // capsule for host-side decoding. Device-only (defmt's linker section).
    #[cfg(all(feature = "userspace-log", target_arch = "riscv32"))]
    EXECUTOR
        .get()
        .spawner()
        .spawn(caliptra_mcu_userlog::drain_task())
        .map_err(|_| log_spawn_error())
        .ok();

    #[cfg(any(
        feature = "test-defmt-logging-mailbox",
        feature = "test-defmt-logging-release",
        feature = "test-defmt-logging-vdm"
    ))]
    defmt_test::emit_test_frames();

    loop {
        EXECUTOR.get().poll();
    }
}

#[cfg(feature = "ocp-lock")]
pub mod ocp_lock_config {
    pub struct AppRuntimeConfig;

    impl caliptra_mcu_romtime::ocp_lock::RuntimeConfig for AppRuntimeConfig {
        fn endorsement_cert_serial_number(&self) -> &[u8; 20] {
            &[0x7F; 20]
        }
    }

    pub static APP_RUNTIME_CONFIG: AppRuntimeConfig = AppRuntimeConfig;
}
