use crate::{audio, tile_batcher::TileBatcher};

use super::config::Config;
use super::Vec2;
use super::GlyphBrush;
use super::page_manager::PageAction;
use super::glutin::event::VirtualKeyCode;
use super::ui::MouseState;

pub trait Page{
    fn enter(&mut self);
    fn tick( &mut self, display: &glium::Display, config: &Config, tile_batcher: &TileBatcher, glyph_brush: &mut GlyphBrush, mouse_state: &MouseState, 
        audio: &audio::Audio, time_in_page: f64, page_actions: &mut Vec<PageAction>  );
    fn mouse_click( &mut self, pressed: bool, pos: Vec2 ) -> PageAction;
    fn mouse_move( &mut self, new_pos: Vec2, audio: &audio::Audio ) -> PageAction;
    fn key_press( &mut self, key: VirtualKeyCode, pressed: bool )  -> PageAction;
}