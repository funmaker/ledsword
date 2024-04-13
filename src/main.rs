#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::error::Error;
use std::time::Instant;
use esp_idf_hal::adc::{AdcChannelDriver, AdcDriver, Atten11dB, Atten6dB};
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::config::Resolution::Resolution10Bit;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::i2s::config::{DataBitWidth, StdConfig};
use esp_idf_hal::i2s::{I2sStdDriver, I2sTx};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::i2s::I2sTxChannel;
use crate::debounce::Debounce;
use crate::led_buffer::LedBuffer;

mod pixel;
mod send;
mod led_buffer;
mod patterns;
mod debounce;

use crate::pixel::Pixel;

fn main() -> Result<(), Box<dyn Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    
    let peripherals = Peripherals::take().unwrap();
    
    let patterns = [
        |id: usize, _: usize, brightness: u8| Pixel::rgb(id as u8 % 2 * 255, id as u8 % 2 * 255, id as u8 % 2 * 255, brightness),
        |id: usize, _: usize, brightness: u8| [
            Pixel::rgb(255, 0, 0, brightness),
            Pixel::rgb(0, 255, 0, brightness),
            Pixel::rgb(0, 0, 255, brightness),
        ][id % 3],
        |id: usize, frame: usize, brightness: u8| {
            if frame % 72 == 71 - id {
                Pixel::rgb(255, 255, 255, brightness)
            } else {
                Pixel::hsv((id + frame) as f32 / 72.0, 1.0, 1.0, brightness)
            }
        },
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_1, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_2, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_3, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_4, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_5, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_6, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_7, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_8, id, frame, brightness),
        |id: usize, frame: usize, brightness: u8| Pixel::image(patterns::IMG_9, id, frame, brightness),
    ];
    
    let mut buffer: LedBuffer<72> = LedBuffer::new();
    
    // let mut adc = AdcDriver::new(
    //     peripherals.adc1,
    //     &Config::default()
    //         .resolution(Resolution10Bit),
    // )?;
    // let mut analog_pin = AdcChannelDriver::<'_, _, Atten11dB<_>>::new(peripherals.pins.gpio36)?;
    // let mut busy_led = PinDriver::output(peripherals.pins.gpio2)?;
    // let mut boot_btn = Debounce::<'_, 10, _, _>::new(PinDriver::input(peripherals.pins.gpio0)?);
    //
    // let i2s = peripherals.i2s0;
    // let data_pin = peripherals.pins.gpio32;
    // let clk_pin = peripherals.pins.gpio25;
    // let ws_pin = peripherals.pins.gpio5;
    //
    // // let mut sender = send::SK9822BitBang::new(data_pin, clk_pin)?;
    // let mut sender = send::SK9822I2s::new(48000, i2s, data_pin, clk_pin, ws_pin)?;
    
    const DELAY_MS: u32 = 0;
    
    let mut frame = 0.0;
    let mut speed = 200.0;
    let mut fps_counter = 0;
    let mut last_update = Instant::now();
    let mut last_print = Instant::now();
    let mut last_sleep = Instant::now();
    
    
    let mut test_btn = Debounce::<'_, 10, _, _>::new(PinDriver::input(peripherals.pins.gpio0)?);
    let mut b_btn = Debounce::<'_, 10, _, _>::new(PinDriver::input(peripherals.pins.gpio32)?);
    let mut a_btn = Debounce::<'_, 10, _, _>::new(PinDriver::input(peripherals.pins.gpio33)?);
    let mut busy_led = PinDriver::output(peripherals.pins.gpio2)?;
    let mut status_led = PinDriver::output(peripherals.pins.gpio15)?;
    
    let data_pin = peripherals.pins.gpio27;
    let clk_pin = peripherals.pins.gpio14;
    let mut sender = send::SK9822BitBang::new(data_pin, clk_pin)?;
    
    loop {
        for (pid, pattern) in patterns.iter().enumerate() {
            println!("pattern {pid}");
        
            while !a_btn.falling_edge() {
                busy_led.set_high()?;
        
                frame += last_update.elapsed().as_secs_f32() * speed;
                fps_counter += 1;
                last_update = Instant::now();
        
                buffer.fill_with(frame as usize, 1, pattern);
        
                let before_send = Instant::now();
        
                sender.send(buffer.to_u32(), 0)?;
        
                busy_led.set_low()?;
        
                let elapsed = last_print.elapsed().as_secs_f32();
                if elapsed > 1.0 {
                    println!("{:.2} FPS, took {} us", fps_counter as f32 / elapsed, before_send.elapsed().as_micros());
        
                    last_print = Instant::now();
                    fps_counter = 0;
                }
        
                if DELAY_MS > 0 {
                    FreeRtos::delay_ms(DELAY_MS);
                } else {
                    let elapsed = last_sleep.elapsed().as_secs_f32();
                    if elapsed > 0.1 {
                        FreeRtos::delay_ms(10);
        
                        last_sleep = Instant::now();
                    }
        
                    // speed = (adc.read(&mut analog_pin)? as f32 / 612.0).max(0.0).powi(2) * 2048.0;
                }
            }
        }
    }
}
