//! # Examples
//!
//! ```ignore
#![doc = include_str!("../examples/sans/main.rs")]
//! ```
//!
//! ![hey there kiddo](https://raw.githubusercontent.com/ZetaNumbers/wasm4-rs/00e582199ed13e59153b808126e4a5ab74267a31/examples/sans/preview.png "sans")

use core::{cell::Cell, marker::PhantomData};

pub use wasm4_common::draw::*;

pub struct Framebuffer(PhantomData<*mut ()>);

impl Framebuffer {
    pub(crate) unsafe fn new_() -> Self {
        Framebuffer(PhantomData)
    }

    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 160;
    pub const BYTE_LENGTH: usize = Self::WIDTH * Self::HEIGHT / 4;

    pub fn as_cell(&self) -> &Cell<[u8; Self::BYTE_LENGTH]> {
        // SAFETY: WASM-4 is single-threaded
        unsafe { &*(wasm4_sys::FRAMEBUFFER.cast::<Cell<[u8; 6400]>>()) }
    }

    pub fn as_cells(&self) -> &[Cell<u8>; Self::BYTE_LENGTH] {
        // SAFETY: WASM-4 is single-threaded
        unsafe { &*(wasm4_sys::FRAMEBUFFER.cast::<[Cell<u8>; 6400]>()) }
    }

    pub fn line(&self, start: [i32; 2], end: [i32; 2]) {
        unsafe { wasm4_sys::line(start[0], start[1], end[0], end[1]) }
    }

    pub fn hline(&self, start: [i32; 2], len: u32) {
        unsafe { wasm4_sys::hline(start[0], start[1], len) }
    }

    pub fn vline(&self, start: [i32; 2], len: u32) {
        unsafe { wasm4_sys::vline(start[0], start[1], len) }
    }

    pub fn oval(&self, start: [i32; 2], shape: [u32; 2]) {
        unsafe { wasm4_sys::oval(start[0], start[1], shape[0], shape[1]) }
    }

    pub fn rect(&self, start: [i32; 2], shape: [u32; 2]) {
        unsafe { wasm4_sys::rect(start[0], start[1], shape[0], shape[1]) }
    }

    pub fn text(&self, text: &str, start: [i32; 2]) {
        unsafe { wasm4_sys::textUtf8(text.as_ptr(), text.len(), start[0], start[1]) }
    }

    pub fn blit(&self, sprite: &impl Blit, start: [i32; 2], transform: BlitTransform) {
        sprite.blit(start, transform, self)
    }

    pub fn replace_palette(&self, palette: [Color; 4]) -> [Color; 4] {
        // SAFETY: Color is `repr(transparent)` over u32
        unsafe { (wasm4_sys::PALETTE as *mut [Color; 4]).replace(palette) }
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct BlitTransform: u32 {
        const FLIP_X = 0b0010;
        const FLIP_Y = 0b0100;
        const ROTATE = 0b1000;
    }
}

pub trait Blit {
    fn blit(&self, start: [i32; 2], transform: BlitTransform, _framebuffer: &Framebuffer);
}

impl Blit for Sprite {
    fn blit(&self, start: [i32; 2], transform: BlitTransform, _framebuffer: &Framebuffer) {
        let flags = self.bpp() as u32 | transform.bits();
        let shape = self.shape();
        unsafe {
            wasm4_sys::blit(
                self.bytes().as_ptr(),
                start[0],
                start[1],
                shape[0],
                shape[1],
                flags,
            )
        }
    }
}

impl<const N: usize> Blit for Sprite<[u8; N]> {
    #[inline(always)]
    fn blit(&self, start: [i32; 2], transform: BlitTransform, _framebuffer: &Framebuffer) {
        Sprite::<[u8]>::blit(self, start, transform, _framebuffer)
    }
}

impl Blit for SpriteView<'_> {
    fn blit(&self, start: [i32; 2], transform: BlitTransform, _framebuffer: &Framebuffer) {
        let flags = self.sprite().bpp() as u32 | transform.bits();
        let shape = self.shape();
        let src_start = self.start();
        let sprite = &self.sprite();

        unsafe {
            wasm4_sys::blitSub(
                sprite.bytes().as_ptr(),
                start[0],
                start[1],
                shape[0],
                shape[1],
                src_start[0],
                src_start[1],
                sprite.shape()[0],
                flags,
            )
        }
    }
}

#[macro_export]
#[cfg(feature = "include-sprites")]
macro_rules! include_sprites {
    ( $( $tt:tt )* ) => {
        $crate::__private::include_sprites_impl! {
            package: $crate,
            input: { $( $tt )* },
        }
    };
}
