use crate::pixel::Pixel;

pub struct LedBuffer<const N: usize>
	where [(); N + 2]: Sized {
	buffer: [Pixel; N + 2],
}

impl<const N: usize> LedBuffer<N>
	where [(); N + 2]: Sized {
	pub fn new() -> Self {
		let mut buffer = [Pixel::end(); N + 2];
		
		buffer[0] = Pixel::start();
		
		Self {
			buffer,
		}
	}
	
	pub fn data(&self) -> &[Pixel] {
		&self.buffer[1..(N + 1)]
	}
	
	pub fn data_mut(&mut self) -> &mut [Pixel] {
		&mut self.buffer[1..(N + 1)]
	}
	
	pub fn fill_with(&mut self, frame: usize, brightness: u8, gen: impl Fn(usize, usize, u8) -> Pixel) {
		for (id, pixel) in self.data_mut().iter_mut().enumerate() {
			*pixel = gen(id, frame, brightness);
		}
	}
	
	pub fn to_u32(&self) -> &[u32] {
		assert_eq!(std::mem::size_of::<Pixel>(), std::mem::size_of::<u32>());
		
		let ptr = self.buffer.as_ptr() as *const u32;
		let size = self.buffer.len();
		
		unsafe {
			std::slice::from_raw_parts(ptr, size)
		}
	}
	
	pub fn to_bytes(&self) -> &[u8] {
		let ptr = self.buffer.as_ptr() as *const u8;
		let size = self.buffer.len() * std::mem::size_of::<Pixel>();
		
		unsafe {
			std::slice::from_raw_parts(ptr, size)
		}
	}
}
