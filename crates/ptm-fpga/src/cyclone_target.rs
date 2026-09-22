// Cyclone target: Intel Cyclone FPGA device parameters and resource estimates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycloneDevice {
    Cyclone10LP,
    CycloneV,
    CycloneIVE,
    CycloneII,
}

impl CycloneDevice {
    pub fn max_le(&self) -> u32 {
        match self {
            CycloneDevice::Cyclone10LP => 150_000,
            CycloneDevice::CycloneV => 500_000,
            CycloneDevice::CycloneIVE => 150_000,
            CycloneDevice::CycloneII => 70_000,
        }
    }

    pub fn max_m9k(&self) -> u32 {
        match self {
            CycloneDevice::Cyclone10LP => 504,
            CycloneDevice::CycloneV => 1220,
            CycloneDevice::CycloneIVE => 504,
            CycloneDevice::CycloneII => 32,
        }
    }

    pub fn max_fmax_mhz(&self) -> u32 {
        match self {
            CycloneDevice::Cyclone10LP => 400,
            CycloneDevice::CycloneV => 500,
            CycloneDevice::CycloneIVE => 400,
            CycloneDevice::CycloneII => 200,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CycloneDevice::Cyclone10LP => "Cyclone 10 LP",
            CycloneDevice::CycloneV => "Cyclone V",
            CycloneDevice::CycloneIVE => "Cyclone IV E",
            CycloneDevice::CycloneII => "Cyclone II",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceEstimate {
    pub logic_elements: u32,
    pub registers: u32,
    pub memory_bits: u32,
    pub m9k_blocks: u32,
    pub dsp_blocks: u32,
    pub io_pins: u32,
    pub fmax_mhz: u32,
}

impl ResourceEstimate {
    pub fn from_table(num_states: u32, num_transitions: u32, tape_size: u32) -> Self {
        let state_bits = (num_states as f64).log2().ceil() as u32;
        let registers = state_bits + (tape_size * 8) * 4 + 32 * 4;
        let state_luts = num_states * 2;
        let trans_luts = num_transitions * 4;
        let logic_elements = state_luts + trans_luts;
        let memory_bits = tape_size * 8 * 4;
        let m9k_blocks = (memory_bits / 9216) + 1;

        ResourceEstimate {
            logic_elements,
            registers,
            memory_bits,
            m9k_blocks,
            dsp_blocks: 0,
            io_pins: 64,
            fmax_mhz: 300,
        }
    }

    pub fn fits_device(&self, device: &CycloneDevice) -> bool {
        self.logic_elements <= device.max_le() && self.m9k_blocks <= device.max_m9k()
    }

    pub fn utilization(&self, device: &CycloneDevice) -> f64 {
        self.logic_elements as f64 / device.max_le() as f64
    }

    pub fn report(&self, device: &CycloneDevice) -> String {
        format!(
            "Resource Estimate for {}:\n  Logic Elements: {}/{} ({:.1}%)\n  Registers: {}\n  Memory: {} bits ({} M9K blocks)\n  DSP Blocks: {}\n  I/O Pins: {}\n  Fmax: {} MHz\n  Fits: {}",
            device.name(),
            self.logic_elements, device.max_le(),
            self.utilization(device) * 100.0,
            self.registers,
            self.memory_bits, self.m9k_blocks,
            self.dsp_blocks,
            self.io_pins,
            self.fmax_mhz,
            if self.fits_device(device) { "YES" } else { "NO" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_simple() {
        let est = ResourceEstimate::from_table(3, 5, 256);
        assert!(est.logic_elements > 0);
        assert!(est.memory_bits > 0);
        assert!(est.fits_device(&CycloneDevice::CycloneV));
    }

    #[test]
    fn cyclone10lp_limits() {
        let dev = CycloneDevice::Cyclone10LP;
        assert_eq!(dev.max_le(), 150_000);
        assert_eq!(dev.max_fmax_mhz(), 400);
    }
}
