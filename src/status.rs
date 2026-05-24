use crate::{flags::Flags, JENT_VERSION};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    pub version: u32,
    pub osr: u32,
    pub flags: u32,
    pub fips_enabled: bool,
    pub ntg1_enabled: bool,
    pub memory_access_enabled: bool,
    pub memory_size: usize,
    pub hash_loop_count: usize,
    pub last_health_failure: u32,
}

impl Status {
    pub(crate) fn new(
        osr: u32,
        flags: Flags,
        memory_size: usize,
        last_health_failure: u32,
    ) -> Self {
        Self {
            version: JENT_VERSION,
            osr,
            flags: flags.bits(),
            fips_enabled: flags.contains(Flags::FORCE_FIPS) || cfg!(feature = "force-fips"),
            ntg1_enabled: flags.contains(Flags::NTG1),
            memory_access_enabled: memory_size > 0,
            memory_size,
            hash_loop_count: flags.hash_loop().count(),
            last_health_failure,
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_json(&self) -> alloc::string::String {
        use alloc::format;
        format!(
            "{{\"version\":{},\"osr\":{},\"flags\":{},\"fips_enabled\":{},\"ntg1_enabled\":{},\"memory_access_enabled\":{},\"memory_size\":{},\"hash_loop_count\":{},\"last_health_failure\":{}}}",
            self.version, self.osr, self.flags, self.fips_enabled, self.ntg1_enabled,
            self.memory_access_enabled, self.memory_size, self.hash_loop_count, self.last_health_failure
        )
    }
}
