
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Pixel {
	red: u8,
	green: u8,
	blue: u8,
	brightness: u8,
}

impl Pixel {
	pub fn rgb(red: u8, green: u8, blue: u8, brightness: u8) -> Self {
		if brightness >= 32 {
			panic!("Brightness can't be greater than 31");
		}
		
		Self {
			brightness: brightness + 0b1110_0000,
			red,
			green,
			blue,
		}
	}
	
	pub fn hsv(h: f32, s: f32, v: f32, brightness: u8) -> Self {
		let i = (h * 6.0).floor();
		let f = h * 6.0 - i;
		let p = v * (1.0 - s);
		let q = v * (1.0 - f * s);
		let t = v * (1.0 - (1.0 - f) * s);
		
		let (r, g, b) = match (i as u32) % 6 {
			0 => (v, t, p),
			1 => (q, v, p),
			2 => (p, v, t),
			3 => (p, q, v),
			4 => (t, p, v),
			5 => (v, p, q),
			_ => (0.0, 0.0, 0.0)
		};
		
		Self::rgb(
			(r * 255.0).floor() as u8,
			(g * 255.0).floor() as u8,
			(b * 255.0).floor() as u8,
			brightness,
		)
	}
	
	pub fn image(bytes: &[u8], x: usize, y: usize, brightness: u8) -> Self {
		let pos = (x + y * 72) * 3 % bytes.len();
		
		Self::rgb(bytes[pos + 0], bytes[pos + 1], bytes[pos + 2], brightness)
	}
	
	pub fn start() -> Self {
		Self {
			brightness: 0,
			red: 0,
			green: 0,
			blue: 0,
		}
	}
	
	pub fn end() -> Self {
		Self {
			brightness: 0xFF,
			red: 0xFF,
			green: 0xFF,
			blue: 0xFF,
		}
	}
}
