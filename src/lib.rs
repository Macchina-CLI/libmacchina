use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_os = "linux", feature = "openwrt"))] {
        mod extra;
        mod openwrt;

        pub type BatteryReadout = openwrt::OpenWrtBatteryReadout;
        pub type KernelReadout = openwrt::OpenWrtKernelReadout;
        pub type MemoryReadout = openwrt::OpenWrtMemoryReadout;
        pub type GeneralReadout = openwrt::OpenWrtGeneralReadout;
        pub type ProductReadout = openwrt::OpenWrtProductReadout;
        pub type PackageReadout = openwrt::OpenWrtPackageReadout;
        pub type NetworkReadout = openwrt::OpenWrtNetworkReadout;
    } else if #[cfg(all(target_os = "linux", not(feature = "openwrt")))] {
        mod extra;
        mod linux;
        mod winman;

        pub type BatteryReadout = linux::LinuxBatteryReadout;
        pub type KernelReadout = linux::LinuxKernelReadout;
        pub type MemoryReadout = linux::LinuxMemoryReadout;
        pub type GeneralReadout = linux::LinuxGeneralReadout;
        pub type ProductReadout = linux::LinuxProductReadout;
        pub type PackageReadout = linux::LinuxPackageReadout;
        pub type NetworkReadout = linux::LinuxNetworkReadout;
    } else if #[cfg(target_os = "macos")] {
        mod extra;
        mod macos;

        pub type BatteryReadout = macos::MacOSBatteryReadout;
        pub type KernelReadout = macos::MacOSKernelReadout;
        pub type MemoryReadout = macos::MacOSMemoryReadout;
        pub type GeneralReadout = macos::MacOSGeneralReadout;
        pub type ProductReadout = macos::MacOSProductReadout;
        pub type PackageReadout = macos::MacOSPackageReadout;
        pub type NetworkReadout = macos::MacOSNetworkReadout;
    } else if #[cfg(target_os = "netbsd")] {
        mod extra;
        mod netbsd;
        mod winman;
        pub mod dirs;

        pub type BatteryReadout = netbsd::NetBSDBatteryReadout;
        pub type KernelReadout = netbsd::NetBSDKernelReadout;
        pub type MemoryReadout = netbsd::NetBSDMemoryReadout;
        pub type GeneralReadout = netbsd::NetBSDGeneralReadout;
        pub type ProductReadout = netbsd::NetBSDProductReadout;
        pub type PackageReadout = netbsd::NetBSDPackageReadout;
        pub type NetworkReadout = netbsd::NetBSDNetworkReadout;
    } else if #[cfg(target_os = "windows")] {
        mod windows;

        pub type BatteryReadout = windows::WindowsBatteryReadout;
        pub type KernelReadout = windows::WindowsKernelReadout;
        pub type MemoryReadout = windows::WindowsMemoryReadout;
        pub type GeneralReadout = windows::WindowsGeneralReadout;
        pub type ProductReadout = windows::WindowsProductReadout;
        pub type PackageReadout = windows::WindowsPackageReadout;
        pub type NetworkReadout = windows::WindowsNetworkReadout;
    } else if #[cfg(target_os = "android")] {
        mod android;
        mod extra;

        pub type BatteryReadout = android::AndroidBatteryReadout;
        pub type KernelReadout = android::AndroidKernelReadout;
        pub type MemoryReadout = android::AndroidMemoryReadout;
        pub type GeneralReadout = android::AndroidGeneralReadout;
        pub type ProductReadout = android::AndroidProductReadout;
        pub type PackageReadout = android::AndroidPackageReadout;
        pub type NetworkReadout = android::AndroidNetworkReadout;
    } else if #[cfg(target_os = "freebsd")] {
        mod extra;
        mod freebsd;
        mod winman;

        pub type BatteryReadout = freebsd::FreeBSDBatteryReadout;
        pub type KernelReadout = freebsd::FreeBSDKernelReadout;
        pub type MemoryReadout = freebsd::FreeBSDMemoryReadout;
        pub type GeneralReadout = freebsd::FreeBSDGeneralReadout;
        pub type ProductReadout = freebsd::FreeBSDProductReadout;
        pub type PackageReadout = freebsd::FreeBSDPackageReadout;
        pub type NetworkReadout = freebsd::FreeBSDNetworkReadout;
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
    pub network: NetworkReadout,
}

#[cfg(feature = "version")]
pub fn version() -> &'static str {
    if let Some(git_sha) = option_env!("VERGEN_GIT_SHA_SHORT") {
        return Box::leak(format!("{} ({})", env!("CARGO_PKG_VERSION"), git_sha).into_boxed_str());
    } else {
        return env!("CARGO_PKG_VERSION");
    }
}

mod shared;
pub mod traits;
