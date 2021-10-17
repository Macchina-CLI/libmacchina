use cfg_if::cfg_if;

#[macro_use]
extern crate lazy_static;

cfg_if! {
    if #[cfg(all(target_os = "linux", feature = "openwrt"))] {
        mod openwrt;

        pub type BatteryReadout = openwrt::OpenWrtBatteryReadout;
        pub type KernelReadout = openwrt::OpenWrtKernelReadout;
        pub type MemoryReadout = openwrt::OpenWrtMemoryReadout;
        pub type GeneralReadout = openwrt::OpenWrtGeneralReadout;
        pub type ProductReadout = openwrt::OpenWrtProductReadout;
        pub type PackageReadout = openwrt::OpenWrtPackageReadout;
    } else if #[cfg(all(target_os = "linux", not(feature = "openwrt")))] {
        mod linux;

        pub type BatteryReadout = linux::LinuxBatteryReadout;
        pub type KernelReadout = linux::LinuxKernelReadout;
        pub type MemoryReadout = linux::LinuxMemoryReadout;
        pub type GeneralReadout = linux::LinuxGeneralReadout;
        pub type ProductReadout = linux::LinuxProductReadout;
        pub type PackageReadout = linux::LinuxPackageReadout;
    } else if #[cfg(target_os = "macos")] {
        mod macos;

        pub type BatteryReadout = macos::MacOSBatteryReadout;
        pub type KernelReadout = macos::MacOSKernelReadout;
        pub type MemoryReadout = macos::MacOSMemoryReadout;
        pub type GeneralReadout = macos::MacOSGeneralReadout;
        pub type ProductReadout = macos::MacOSProductReadout;
        pub type PackageReadout = macos::MacOSPackageReadout;
    } else if #[cfg(target_os = "netbsd")] {
        mod netbsd;

        pub type BatteryReadout = netbsd::NetBSDBatteryReadout;
        pub type KernelReadout = netbsd::NetBSDKernelReadout;
        pub type MemoryReadout = netbsd::NetBSDMemoryReadout;
        pub type GeneralReadout = netbsd::NetBSDGeneralReadout;
        pub type ProductReadout = netbsd::NetBSDProductReadout;
        pub type PackageReadout = netbsd::NetBSDPackageReadout;
    } else if #[cfg(target_os = "windows")] {
        mod windows;

        pub type BatteryReadout = windows::WindowsBatteryReadout;
        pub type KernelReadout = windows::WindowsKernelReadout;
        pub type MemoryReadout = windows::WindowsMemoryReadout;
        pub type GeneralReadout = windows::WindowsGeneralReadout;
        pub type ProductReadout = windows::WindowsProductReadout;
        pub type PackageReadout = windows::WindowsPackageReadout;
    } else if #[cfg(target_os = "android")] {
        mod android;

        pub type BatteryReadout = android::AndroidBatteryReadout;
        pub type KernelReadout = android::AndroidKernelReadout;
        pub type MemoryReadout = android::AndroidMemoryReadout;
        pub type GeneralReadout = android::AndroidGeneralReadout;
        pub type ProductReadout = android::AndroidProductReadout;
        pub type PackageReadout = android::AndroidPackageReadout;
    } else if #[cfg(target_os = "freebsd")] {
        mod freebsd;

        pub type BatteryReadout = freebsd::FreeBSDBatteryReadout;
        pub type KernelReadout = freebsd::FreeBSDKernelReadout;
        pub type MemoryReadout = freebsd::FreeBSDMemoryReadout;
        pub type GeneralReadout = freebsd::FreeBSDGeneralReadout;
        pub type ProductReadout = freebsd::FreeBSDProductReadout;
        pub type PackageReadout = freebsd::FreeBSDPackageReadout;
    } else {
        compiler_error!("This platform is currently not supported by libmacchina.");
    }
}

pub struct Readouts {
    pub battery: BatteryReadout,
    pub kernel: KernelReadout,
    pub memory: MemoryReadout,
    pub general: GeneralReadout,
    pub product: ProductReadout,
    pub packages: PackageReadout,
}

pub fn version() -> &'static str {
    if let Some(git_sha) = option_env!("VERGEN_GIT_SHA_SHORT") {
        // return concat!(
        //     env!("CARGO_PKG_VERSION"),
        //     " (",
        //     env!("VERGEN_GIT_SHA_SHORT"), // fails
        //     ")");

        // Leaks memory.
        return Box::leak(format!("{} ({})", env!("CARGO_PKG_VERSION"), git_sha).into_boxed_str());
    } else {
        return env!("CARGO_PKG_VERSION");
    }
}

pub mod extra;
mod shared;
pub mod traits;
