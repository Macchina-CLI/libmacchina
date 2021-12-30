use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_os = "linux", feature = "openwrt"))] {
        mod extra;
        mod openwrt;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = openwrt::OpenWrtBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = openwrt::OpenWrtKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = openwrt::OpenWrtMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = openwrt::OpenWrtGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = openwrt::OpenWrtProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = openwrt::OpenWrtPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = openwrt::OpenWrtNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type GraphicalReadout = openwrt::OpenWrtGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type ProcessorReadout = openwrt::OpenWrtProcessorReadout;
    } else if #[cfg(all(target_os = "linux", not(feature = "openwrt")))] {
        #[cfg(feature = "graphical")]
        mod winman;
        mod extra;
        mod linux;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = linux::battery::LinuxBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = linux::kernel::LinuxKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = linux::memory::LinuxMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = linux::general::LinuxGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = linux::product::LinuxProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = linux::package::LinuxPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = linux::network::LinuxNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type GraphicalReadout = linux::graphical::LinuxGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type ProcessorReadout = linux::processor::LinuxProcessorReadout;
    } else if #[cfg(target_os = "macos")] {
        mod extra;
        mod macos;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = macos::MacOSBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = macos::MacOSKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = macos::MacOSMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = macos::MacOSGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = macos::MacOSProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = macos::MacOSPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = macos::MacOSNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type GraphicalReadout = macos::MacOSGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type ProcessorReadout = macos::MacOSProcessorReadout;
    } else if #[cfg(target_os = "netbsd")] {
        #[cfg(feature = "graphical")]
        mod winman;
        mod extra;
        mod netbsd;
        pub mod dirs;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = netbsd::NetBSDBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = netbsd::NetBSDKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = netbsd::NetBSDMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = netbsd::NetBSDGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = netbsd::NetBSDProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = netbsd::NetBSDPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = netbsd::NetBSDNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type GraphicalReadout = netbsd::NetBSDGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type ProcessorReadout = netbsd::NetBSDProcessorReadout;
    } else if #[cfg(target_os = "windows")] {
        mod windows;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = windows::WindowsBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = windows::WindowsKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = windows::WindowsMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = windows::WindowsGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = windows::WindowsProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = windows::WindowsPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = windows::WindowsNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type GraphicalReadout = windows::WindowsGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type ProcessorReadout = windows::WindowsProcessorReadout;
    } else if #[cfg(target_os = "android")] {
        mod android;
        mod extra;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = android::AndroidBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = android::AndroidKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = android::AndroidMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = android::AndroidGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = android::AndroidProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = android::AndroidPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = android::AndroidNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type GraphicalReadout = android::AndroidGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type ProcessorReadout = android::AndroidProcessorReadout;
    } else if #[cfg(target_os = "freebsd")] {
        #[cfg(feature = "graphical")]
        mod winman;
        mod extra;
        mod freebsd;

        #[cfg(feature = "battery")]
        pub type BatteryReadout = freebsd::FreeBSDBatteryReadout;
        #[cfg(feature = "kernel")]
        pub type KernelReadout = freebsd::FreeBSDKernelReadout;
        #[cfg(feature = "memory")]
        pub type MemoryReadout = freebsd::FreeBSDMemoryReadout;
        #[cfg(feature = "general")]
        pub type GeneralReadout = freebsd::FreeBSDGeneralReadout;
        #[cfg(feature = "product")]
        pub type ProductReadout = freebsd::FreeBSDProductReadout;
        #[cfg(feature = "package")]
        pub type PackageReadout = freebsd::FreeBSDPackageReadout;
        #[cfg(feature = "network")]
        pub type NetworkReadout = freebsd::FreeBSDNetworkReadout;
        #[cfg(feature = "graphical")]
        pub type NetworkReadout = freebsd::FreeBSDGraphicalReadout;
        #[cfg(feature = "processor")]
        pub type NetworkReadout = freebsd::FreeBSDProcessorReadout;
    } else {
        compiler_error!("This platform is currently not supported by libmacchina.");
    }
}

pub struct Readouts {
    #[cfg(feature = "battery")]
    pub battery: BatteryReadout,
    #[cfg(feature = "kernel")]
    pub kernel: KernelReadout,
    #[cfg(feature = "memory")]
    pub memory: MemoryReadout,
    #[cfg(feature = "general")]
    pub general: GeneralReadout,
    #[cfg(feature = "product")]
    pub product: ProductReadout,
    #[cfg(feature = "package")]
    pub packages: PackageReadout,
    #[cfg(feature = "network")]
    pub network: NetworkReadout,
    #[cfg(feature = "graphical")]
    pub graphical: GraphicalReadout,
    #[cfg(feature = "processor")]
    pub processor: ProcessorReadout,
}

#[cfg(feature = "version")]
pub fn version() -> &'static str {
    if let Some(git_sha) = option_env!("VERGEN_GIT_SHA_SHORT") {
        Box::leak(format!("{} ({})", env!("CARGO_PKG_VERSION"), git_sha).into_boxed_str())
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

mod shared;
pub mod enums;
pub mod traits;
