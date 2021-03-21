use crate::{Vertex, page_manager};
use glium_glyph::GlyphBrush;

use super::glutin::event::VirtualKeyCode;
use super::glium::Surface;
use super::Vec2;
use super::Vec4;
use super::Level;
use super::tile_batcher::TileBatcher;
use super::config::Config;
use super::page;
use super::page_manager::{PageAction, PageName};
use super::ui::{button,MouseState};
use super::render_level;
use super::audio;
use super::level::Tile;

#[derive(Debug, PartialEq)]
enum MainMenuState{
    Showing,
    ChangingPage( PageAction, f64 ),
}

pub struct MainMenuPage{
    level: Level,
    game_state: MainMenuState,
}

impl MainMenuPage{
    pub fn new( ) -> MainMenuPage {
        let level = Level::load_level(0);
        return MainMenuPage{ level, game_state: MainMenuState::Showing};
    }
    
    fn ui(&mut self, tile_batcher: &TileBatcher, vertices: &mut Vec<Vertex>, glyph_brush: &mut GlyphBrush, config: &Config, mouse_state: &MouseState, time_in_page: f64 ) {
        let button_width = config.width() as f32 * 0.8f32;
        let button_height = 80.0f32;
        let button_left = ( config.width() as f32 - button_width ) / 2.0;

        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 800.0 ),Vec2::new( button_width, button_height ), "Play", &mouse_state, 
            &mut || self.game_state = MainMenuState::ChangingPage( PageAction::VisitPage(PageName::Game), time_in_page));
        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 600.0 ),Vec2::new( button_width, button_height ), "Settings", &mouse_state,
             &mut || self.game_state = MainMenuState::ChangingPage( PageAction::VisitPage(PageName::Settings), time_in_page));
        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 400.0 ),Vec2::new( button_width, button_height ), "Exit", &mouse_state, 
            &mut || self.game_state = MainMenuState::ChangingPage(  PageAction::Exit, time_in_page ));
    }
}

impl page::Page for MainMenuPage {
    fn enter(&mut self){
        self.game_state = MainMenuState::Showing;
    }

    fn tick( &mut self, display: &glium::Display, config: &Config, tile_batcher: &TileBatcher, glyph_brush: &mut GlyphBrush, mouse_state: &MouseState, audio: &audio::Audio, time_in_page: f64,
            page_actions: &mut Vec<PageAction> )  {

        let mut vertices: Vec<Vertex> = Vec::new();
        render_level::render_level( config, tile_batcher, &self.level, &mut vertices, 0.0);

        self.ui(tile_batcher, &mut vertices, glyph_brush, config, mouse_state, time_in_page);

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
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            scissor: Some( page_manager::get_scissor_rectangle(display)),
            ..Default::default()
        };
        
        target.draw(&vertex_buffer, &indices, &tile_batcher.program, &uniforms, &draw_params).unwrap();
        glyph_brush.draw_queued_with_transform(view_matrix_c, display, &mut target);

        if let MainMenuState::ChangingPage( page_action , time_started) = self.game_state  {
            let mut vertices: Vec<Vertex> = Vec::new();
            let tile: u8 = Tile::Solid.into();
            let src = Vec2::new( (tile%14) as f32 / 16.0f32, 1.0-((tile/14) as f32 /16f32) );
       
            let back_depth = (time_in_page - time_started) as f32 * 1.7f32;
            tile_batcher.tile_color(&mut vertices, &Vec2::new( 0.0, config.height() as f32), &Vec2::new( config.width() as f32, config.height() as f32 ) ,&src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 1.0, 1.0, 1.0, back_depth ));

            let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
            target.draw(&vertex_buffer, &indices, &tile_batcher.program, &uniforms, &draw_params).unwrap();

            if back_depth > 1.0 {
                page_actions.push( page_action );
                self.game_state = MainMenuState::Showing;
            }
        }

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