use std::time::Instant;
use esp_idf_hal::gpio::{InputMode, Pin, PinDriver};

pub struct Debounce<'d, const MS: u128, T, Mode>
	where T: Pin,
	      Mode: InputMode {
	pub inner: PinDriver<'d, T, Mode>,
	last_change: Instant,
	last_value: bool,
}

impl<'d, const MS: u128, T, Mode> Debounce<'d, MS, T, Mode>
	where T: Pin,
	      Mode: InputMode {
	pub fn new(inner: PinDriver<'d, T, Mode>) -> Self {
		Self {
			last_change: Instant::now(),
			last_value: inner.is_high(),
			inner,
		}
	}
	
	pub fn raising_edge(&mut self) -> bool {
		let changed = self.update();
		
		changed && self.last_value
	}
	
	pub fn falling_edge(&mut self) -> bool {
		let changed = self.update();
		
		changed && !self.last_value
	}
	
	pub fn is_low(&mut self) -> bool {
		self.update();
		
		!self.last_value
	}
	
	pub fn is_high(&mut self) -> bool {
		self.update();
		
		self.last_value
	}
	
	fn update(&mut self) -> bool {
		if self.last_change.elapsed().as_millis() > MS {
			let current_value = self.inner.is_high();
			
			if current_value != self.last_value {
				self.last_value = current_value;
				self.last_change = Instant::now();
				
				return true;
			}
		}
		
		false
	}
}
