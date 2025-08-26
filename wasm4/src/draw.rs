//! # Examples
//!
//! ```ignore
#![doc = include_str!("../examples/sans/main.rs")]
//! ```
//!
//! ![hey there kiddo](https://raw.githubusercontent.com/ZetaNumbers/wasm4-rs/00e582199ed13e59153b808126e4a5ab74267a31/examples/sans/preview.png "sans")

use core::{cell::Cell, marker::PhantomData};

pub use wasm4_common::draw::*;

pub struct Framebuffer {
    phantom: PhantomData<*mut ()>,
    foreground: DrawIndex,
    background: DrawIndex,
    fill: DrawIndex,
    stroke: DrawIndex,
}

impl Framebuffer {
    pub(crate) unsafe fn new_() -> Self {
        Framebuffer {
            phantom: PhantomData,
            foreground: DrawIndex::Fourth,
            background: DrawIndex::First,
            fill: DrawIndex::Second,
            stroke: DrawIndex::Fourth,
        }
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
        unsafe {
            wasm4_sys::DRAW_COLORS.write(self.stroke as u16);
            wasm4_sys::line(start[0], start[1], end[0], end[1]);
        }
    }

    pub fn hline(&self, start: [i32; 2], len: u32) {
        unsafe {
            wasm4_sys::DRAW_COLORS.write(self.stroke as u16);
            wasm4_sys::hline(start[0], start[1], len);
        }
    }

    pub fn vline(&self, start: [i32; 2], len: u32) {
        unsafe {
            wasm4_sys::DRAW_COLORS.write(self.stroke as u16);
            wasm4_sys::vline(start[0], start[1], len);
        }
    }

    pub fn oval(&self, start: [i32; 2], shape: [u32; 2]) {
        unsafe {
            wasm4_sys::DRAW_COLORS.write(((self.stroke as u16) << 4) | (self.fill as u16));
            wasm4_sys::oval(start[0], start[1], shape[0], shape[1])
        }
    }

    pub fn rect(&self, start: [i32; 2], shape: [u32; 2]) {
        unsafe {
            wasm4_sys::DRAW_COLORS.write(((self.stroke as u16) << 4) | (self.fill as u16));
            wasm4_sys::rect(start[0], start[1], shape[0], shape[1])
        }
    }

    pub fn text(&self, text: &str, start: [i32; 2]) {
        unsafe {
            wasm4_sys::DRAW_COLORS
                .write(((self.background as u16) << 4) | (self.foreground as u16));
            wasm4_sys::textUtf8(text.as_ptr(), text.len(), start[0], start[1]);
        }
    }

    pub fn blit(&self, sprite: &impl Blit, start: [i32; 2], transform: BlitTransform) {
        sprite.blit(start, transform, self)
    }

    pub fn flip_palette(&self) {
        // SAFETY: only mut reference because WASM-4 is single-threaded
        let colors = unsafe { &mut *wasm4_sys::PALETTE };

        (colors[0], colors[3]) = (colors[3], colors[0]);
        (colors[1], colors[2]) = (colors[2], colors[1]);
    }

    pub fn reset_palette(&self) {
        // SAFETY: only mut reference because WASM-4 is single-threaded
        let colors = unsafe { &mut *wasm4_sys::PALETTE };

        colors[0] = 0xE0F8CF;
        colors[1] = 0x86C06C;
        colors[2] = 0x306850;
        colors[3] = 0x071821;
    }

    pub fn palette(&self) -> Palette {
        unsafe { (wasm4_sys::PALETTE as *mut Palette).read() }
    }

    pub fn set_palette(&self, palette: Palette) {
        // SAFETY: Color is `repr(transparent)` over u32
        unsafe { (wasm4_sys::PALETTE as *mut Palette).write(palette) }
    }

    pub fn set_draw_indices(&self, indices: DrawIndices) {
        unsafe {
            wasm4_sys::DRAW_COLORS.write(indices.into());
        }
    }

    pub fn set_foreground(&mut self, index: DrawIndex) {
        self.foreground = index;
    }

    pub fn set_background(&mut self, index: DrawIndex) {
        self.background = index;
    }

    pub fn set_fill(&mut self, index: DrawIndex) {
        self.fill = index;
    }

    pub fn set_stroke(&mut self, index: DrawIndex) {
        self.stroke = index;
    }

    pub fn reset_draw_indices(&self) {
        self.set_draw_indices(DrawIndices::DEFAULT);
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
    fn blit(&self, start: [i32; 2], transform: BlitTransform, framebuffer: &Framebuffer);

    /// Same as [Blit::blit] but without zero [BlitTransform] (no flip/rotation).
    fn blitz(&self, start: [i32; 2], framebuffer: &Framebuffer) {
        self.blit(start, BlitTransform::empty(), framebuffer);
    }
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
    fn blit(&self, start: [i32; 2], transform: BlitTransform, framebuffer: &Framebuffer) {
        Sprite::<[u8]>::blit(self, start, transform, framebuffer)
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

#[derive(Clone, Copy)]
pub struct DrawIndexed<'a, T: ?Sized> {
    item: &'a T,
    draw_indices: DrawIndices,
}

impl<'a, T: ?Sized> DrawIndexed<'a, T> {
    pub const fn from(item: &'a T, draw_indices: DrawIndices) -> DrawIndexed<'a, T> {
        Self { item, draw_indices }
    }

    pub const fn with_draw_indices(&self, draw_indices: DrawIndices) -> DrawIndexed<'a, T> {
        Self {
            item: self.item,
            draw_indices,
        }
    }
}

impl<'a, T: Blit> Blit for DrawIndexed<'a, T>
where
    T: ?Sized,
{
    fn blit(&self, start: [i32; 2], transform: BlitTransform, framebuffer: &Framebuffer) {
        framebuffer.set_draw_indices(self.draw_indices);
        self.item.blit(start, transform, framebuffer);
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

/**
Creates a sprite from an inline text representation.

# Format
You can use any characters (`char`s) to define different indices.
Every character (except leading/trailing spaces) has to be
repeated exactly twice. We do that to counter typical aspect ratios
of mono-spaced characters in text editors.

A single leading new-line is mandatory and so are the new-lines after
each line (including the last one).

Width, height, bits per pixel, and buffer size are auto-detected.
Spaces are ignored so you can use indentation inside the raw string literal.

Indices are set in order of appearance of the different characters and
can not be changed. Use [DrawIndices] before drawing to map to different
[Palette] colors.

# Examples
```
const GRID: &Sprite = text_sprite!(
    r#"
    0011
    2233
    "#
);

const SMILEY: &Sprite = text_sprite!(
    r#"
    ....▒▒▒▒▒▒▒▒....
    ..▒▒▒▒▒▒▒▒▒▒▒▒..
    ▒▒▒▒██▒▒▒▒██▒▒▒▒
    ▒▒▒▒▓▓▒▒▒▒▓▓▒▒▒▒
    ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
    ▒▒▒▒██▒▒▒▒██▒▒▒▒
    ..▒▒▒▒████▒▒▒▒..
    ....▒▒▒▒▒▒▒▒....
    "#
);
```

*/
#[macro_export]
macro_rules! text_sprite {
    ($data:literal) => {
        const {
            &$crate::__private::inline_sprite::sprite_builder::<
                { $crate::__private::inline_sprite::sprite_calc($data).1 },
            >($data)
        };
    };
}
