use crate::{Vertex};
use glium_glyph::GlyphBrush;

use super::glutin::event::VirtualKeyCode;
use super::glium::Surface;
use super::Vec2;
use super::Level;
use super::tile_batcher::TileBatcher;
use super::config::Config;
use super::page;
use super::page_manager::{PageAction, PageName};
use super::page_manager;
use super::ui::{button,multi_selector,MouseState};
use super::render_level;
use super::audio;

pub struct SettingsPage{
    level: Level,
}

impl SettingsPage{
    pub fn new( ) -> SettingsPage {
        let level = Level::load_level(0);
        return SettingsPage{ level };
    }
    
    fn ui(&mut self, tile_batcher: &TileBatcher, vertices: &mut Vec<Vertex>, glyph_brush: &mut GlyphBrush, config: &Config, mouse_state: &MouseState, page_actions: &mut Vec<PageAction>) {
        let button_width = config.width() as f32 * 0.8f32;
        let button_height = 80.0f32;
        let button_left = ( config.width() as f32 - button_width ) / 2.0;

        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 800.0 ),Vec2::new( button_width, button_height ), "Back", &mouse_state, 
            &mut || page_actions.push(PageAction::Back));
        
        multi_selector(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 600.0 ),Vec2::new( button_width, 120.0 ), mouse_state,
            "Window Mode", &vec![ "Full Screen", "Windowed"], if config.fullscreen() {0} else { 1},
            &mut |idx | page_actions.push( PageAction::SetFullScreen(if idx==0 {true} else {false})) );
    }
}

impl page::Page for SettingsPage {
    fn enter(&mut self){
    }

    fn tick( &mut self, display: &glium::Display, config: &Config, tile_batcher: &TileBatcher, glyph_brush: &mut GlyphBrush, mouse_state: &MouseState, audio: &audio::Audio, time_in_page: f64,
            page_actions: &mut Vec<PageAction> ) {
        let mut vertices: Vec<Vertex> = Vec::new();
        render_level::render_level( config, tile_batcher, &self.level, &mut vertices, 0.0);

        self.ui(tile_batcher, &mut vertices, glyph_brush, config, mouse_state, page_actions);

        let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let view_matrix_c: [[f32; 4]; 4] = page_manager::get_view_matrix(display);
        let uniforms = glium::uniform! {
            matrix: view_matrix_c,
            tex: glium::uniforms::Sampler::new(&tile_batcher.texture).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).
            minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
        };

        let draw_params = glium::draw_parameters::DrawParameters{
            blend: glium::draw_parameters::Blend{
                color: glium::draw_parameters::BlendingFunction::Addition{
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::OneMinusSourceAlpha
                },
                alpha: glium::draw_parameters::BlendingFunction::Addition{
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::OneMinusSourceAlpha
                },
                constant_value: (0.0, 0.0, 0.0, 0.0 )
            },
            scissor: Some( page_manager::get_scissor_rectangle(display)),
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };

        target.draw(&vertex_buffer, &indices, &tile_batcher.program, &uniforms, &draw_params).unwrap();
        glyph_brush.draw_queued_with_transform(view_matrix_c, display, &mut target);
        target.finish().unwrap();   

    }
    fn mouse_click( &mut self, pressed: bool, pos: Vec2 ) -> PageAction {
        return PageAction::None;
    }

    fn mouse_move( &mut self, new_pos: Vec2, audio: &audio::Audio ) -> PageAction{
        return PageAction::None;
    }

    fn key_press( &mut self, key: VirtualKeyCode, pressed: bool ) -> PageAction{ 
        return PageAction::None;
    }

}