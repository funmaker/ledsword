use std::time::Duration;
use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio::{AnyIOPin, InputPin, Output, OutputPin, PinDriver};
use esp_idf_hal::i2s::{I2s, I2sStdDriver, I2sTx, I2sTxChannel, I2sTxSupported};
use esp_idf_hal::i2s::config::{Config, DataBitWidth, SlotMode, StdClkConfig, StdConfig, StdGpioConfig, StdSlotConfig, StdSlotMask};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_sys::EspError;

pub struct SK9822BitBang<'d, 'c, D, C>
	where D: OutputPin,
	      C: OutputPin {
	data: PinDriver<'d, D, Output>,
	clk: PinDriver<'c, C, Output>,
}

impl<'d, 'c, D, C> SK9822BitBang<'d, 'c, D, C>
	where D: OutputPin,
	      C: OutputPin {
	pub fn new(data: impl Peripheral<P = D> + 'd,
	           clk: impl Peripheral<P = C> + 'c)
	           -> Result<Self, EspError> {
		Ok(SK9822BitBang {
			data: PinDriver::output(data)?,
			clk: PinDriver::output(clk)?,
		})
	}
	
	pub fn send(&mut self, data: &[u32], delay_us: u32) -> Result<(), EspError> {
		for byte in data {
			for bit in (0..32).rev() {
				self.clk.set_low()?;
				
				if byte & (1 << bit) != 0 {
					self.data.set_high()?;
				} else {
					self.data.set_low()?;
				}
				
				if delay_us != 0 { Delay::delay_us(delay_us); }
				
				self.clk.set_high()?;
				
				if delay_us != 0 { Delay::delay_us(delay_us); }
			}
		}
		
		self.clk.set_high()?;
		self.data.set_high()?;
		
		Ok(())
	}
}


pub struct SK9822I2s<'i> {
	i2s: I2sStdDriver<'i, I2sTx>
}

impl<'i> SK9822I2s<'i> {
	pub fn new(sample_rate: u32,
	           i2s: impl Peripheral<P = impl I2s> + 'i,
	           data: impl Peripheral<P = impl OutputPin> + 'i,
	           clk: impl Peripheral<P = impl OutputPin + InputPin> + 'i,
	           ws: impl Peripheral<P = impl OutputPin + InputPin> + 'i)
	           -> Result<Self, EspError> {
		let mut i2s = I2sStdDriver::<I2sTx>::new_tx(i2s,
		                                            StdConfig::new(
			                                            Config::default().frames(37),
			                                            StdClkConfig::from_sample_rate_hz(sample_rate),
			                                            StdSlotConfig::msb_slot_default(DataBitWidth::Bits32, SlotMode::Mono)
				                                            .slot_mode_mask(SlotMode::Mono, StdSlotMask::Both),
			                                            StdGpioConfig::default(),
		                                            ),
		                                            clk,
		                                            Some(data),
		                                            AnyIOPin::none(),
		                                            ws)?;
		
		i2s.tx_enable()?;
		
		Ok(Self { i2s })
	}
	
	pub fn send(&mut self, data: &[u8], _: u32) -> Result<(), EspError> {
		let wrote = self.i2s.write(data, Duration::from_secs(1))?;
		
		Ok(())
	}
}

