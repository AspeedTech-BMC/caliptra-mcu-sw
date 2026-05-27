// Licensed under the Apache-2.0 license
use arrayvec::ArrayVec;
use caliptra_mcu_capsules_runtime::bootdev::hil::ImageFlash;
use caliptra_mcu_capsules_runtime::bootdev::BootDevCapsule;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::{capabilities, create_capability};

#[macro_export]
macro_rules! bootdev_component_static {
    () => {{
        use caliptra_mcu_capsules_runtime::bootdev::BootDevCapsule;
        let capsule = kernel::static_buf!(BootDevCapsule<'static>);
        (capsule,)
    }};
}

pub struct BootDevComponent<const N: usize> {
    flash_devs: ArrayVec<&'static dyn ImageFlash, N>,
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<const N: usize> BootDevComponent<N> {
    pub fn new(
        flash_devs: ArrayVec<&'static dyn ImageFlash, N>,
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> Self {
        Self {
            flash_devs,
            board_kernel,
            driver_num,
        }
    }
}

impl<const N: usize> Component for BootDevComponent<N> {
    type StaticInput = (&'static mut MaybeUninit<BootDevCapsule<'static>>,);
    type Output = &'static BootDevCapsule<'static>;

    fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let capsule = static_input.0.write(BootDevCapsule::new(
            self.flash_devs.as_slice(),
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        capsule
    }
}
