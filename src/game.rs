
use super::{Vertex, level::Tile, page_manager::PageName};
use super::glutin::event::VirtualKeyCode;

use super::glium::Surface;
use super::{Vec2,Vec4};
use super::Level;
use super::level::TileEffect;
use super::tile_batcher::*;
use super::config;
use super::GlyphBrush;
use super::{page,page_manager::PageAction};
use super::ui::{button,static_text, MouseState};
use super::page_manager;
use super::render_level;
use super::audio;

#[derive(Clone, Copy)]
struct TileMove{
    tile: Tile,
    map_x: u32,
    map_y: u32,
    grab_cursor_pos: Vec2,
    last_cursor_pos:Vec2
}

#[derive(Debug, PartialEq)]
enum GameState{
    ShowingNewLevel( f64 ),
    Playing,
    ShowingSolution( f64 ),
    InGameMenu,
    ChangingPage( PageAction, f64 )
}

pub struct GamePage{
    level_no: u32,
    game_state: GameState,

    level: Level,
    tile_move: Option<TileMove>,
    
    last_jewel_ray_count: u32,           // how many tiles crossed by the ray on the last frame
    last_map_pos: Option<(u32,u32)>
}

impl GamePage{
    pub fn new( config: &config::Config ) -> GamePage {
        let level_no = config.max_level();
        let level = Level::load_level(level_no);
        return GamePage{ level_no, game_state: GameState::ShowingNewLevel( 0.0), level, tile_move: None, last_jewel_ray_count: 0, last_map_pos: None };
    }

    fn ui(&mut self, tile_batcher: &TileBatcher, vertices: &mut Vec<Vertex>, glyph_brush: &mut GlyphBrush, config: &config::Config, mouse_state: &MouseState, page_actions: &mut Vec<PageAction>) {
        let button_width = config.width() as f32 * 0.8f32;
        let button_height = 80.0f32;
        let button_left = ( config.width() as f32 - button_width ) / 2.0;

        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 900.0 ),Vec2::new( button_width, button_height ), "Continue", &mouse_state, 
        &mut ||self.game_state = GameState::Playing );
        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 700.0 ),Vec2::new( button_width, button_height ), "Settings", &mouse_state, 
        &mut || page_actions.push( PageAction::VisitPage( PageName::Settings)));
        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 500.0 ),Vec2::new( button_width, button_height ), "Main Menu", &mouse_state, 
        &mut || page_actions.push( PageAction::Back));
        button(tile_batcher, vertices, glyph_brush, config, Vec2::new( button_left, 300.0 ),Vec2::new( button_width, button_height ), "Exit Game", &mouse_state, 
        &mut || page_actions.push( PageAction::Exit));
    }

    // Convert the position into a level map coordinate
    fn to_level_pos( &self, pos: &Vec2 ) -> Option<(u32,u32)> {
        let map_x = ( pos.x / 64.0 ) as u32;
        let map_y = ( pos.y / 64.0 ) as u32;
        if map_x < self.level.width && map_y < self.level.height {
            return Some( (map_x, map_y) );
        } else {
            return None;
        }
    }

}


impl page::Page for GamePage {
    
    fn enter(&mut self) {
        self.last_jewel_ray_count = 0;
        self.game_state = GameState::ShowingNewLevel( 0.0);
        self.level = Level::load_level(self.level_no);
        self.level.tile_movable_effect(TileEffect::Hide );
    }

    fn tick( &mut self, display: &glium::Display, config: &config::Config, tile_batcher: &TileBatcher, glyph_brush: &mut GlyphBrush, mouse_state: &MouseState, audio: &audio::Audio, time_in_page: f64, 
            page_actions: &mut Vec<PageAction> ) {

//        println!( "State {:?}", self.game_state );
        let jewel_ray_count;
        if let GameState::ShowingSolution( solution_start ) = self.game_state {
            let secs_in_state = time_in_page - solution_start;
            jewel_ray_count = self.level.update_ray( ( secs_in_state / 0.05 ) as usize, time_in_page );
            if jewel_ray_count > self.last_jewel_ray_count {
                audio.play_sound(audio::SoundEffect::GemSolved);
            }                
        } else {
            if self.tile_move.is_some() {
                let tile_move = self.tile_move.take().unwrap();
                self.level.set_front_tile( tile_move.map_x, tile_move.map_y, tile_move.tile );
                
                jewel_ray_count = self.level.update_ray( 500, time_in_page );
                if jewel_ray_count > self.last_jewel_ray_count {
                    audio.play_sound(audio::SoundEffect::Gem);
                }                
                self.level.set_front_tile( tile_move.map_x, tile_move.map_y, Tile::EmptyPiece );
                self.tile_move = Some( tile_move);
            } else {
                jewel_ray_count = self.level.update_ray( 500, time_in_page );
                let solved = jewel_ray_count == self.level.count_jewels();
                if self.game_state == GameState::Playing && solved {
                    self.game_state = GameState::ShowingSolution( time_in_page );
                }
            }
        }
        self.last_jewel_ray_count = jewel_ray_count;
        let map_pos = self.to_level_pos(&mouse_state.pos);
        if let Some( (map_x, map_y ) ) = map_pos {
            if self.level.front_tile(map_x, map_y).is_movable() {
                let tickle_piece = match self.last_map_pos{
                    None => true,
                    Some( (old_x, old_y) ) => {
                        !(old_x == map_x && old_y == map_y)
                    }
                };
                if tickle_piece {
                    self.level.set_effect( map_x, map_y, TileEffect::Punch( time_in_page as f32, Vec2::new( 5.0, 5.0 )));
                }
            }
        }
        self.last_map_pos = map_pos;

        let mut vertices: Vec<Vertex> = Vec::new();
        render_level::render_level( config, tile_batcher, &self.level, &mut vertices, time_in_page);

        let mut glow_vertices: Vec<Vertex> = Vec::new();
        render_level::render_level_glows( config, tile_batcher, &self.level, &mut glow_vertices, time_in_page);

        let qtr_pixel=1.02f32/( 16f32 * 64f32 * 4f32 );
        let half_pixel= 1.02f32/( 16f32 * 64f32 * 2f32 );
        if self.tile_move.is_some() {
            let tile_move = self.tile_move.as_ref().unwrap();
            let delta = tile_move.last_cursor_pos - tile_move.grab_cursor_pos;
            let x = tile_move.map_x as f32 * 64.0+delta.x;
            let y = tile_move.map_y as f32 * 64.0+delta.y;
            let tile_idx: u8 = tile_move.tile.into();
            let src = Vec2::new( (tile_idx%14) as f32 / 16.0f32+qtr_pixel, 1.0-((tile_idx/14) as f32 /16f32)-qtr_pixel );
            let ( scaled_pos, scaled_size ) = scale( &Vec2::new( x, config.height() as f32 - y ), &Vec2::new( 64.0, 64.0 ), 1.5f32);
            tile_batcher.tile(&mut vertices,&scaled_pos, &scaled_size, &src, &Vec2::new( 1.0/16.0-half_pixel, 1.0/16.0-half_pixel ));
        }

        if self.game_state == GameState::InGameMenu {
            self.ui(tile_batcher, &mut vertices, glyph_brush, config, mouse_state, page_actions);
        }

        if let GameState::ShowingNewLevel( level_start) = self.game_state {
            let button_width = config.width() as f32 * 0.6f32;
            let button_height = 80.0f32;
            let button_left = ( config.width() as f32 - button_width ) / 2.0;
            let level_message = "Level ".to_owned() + &self.level_no.to_string();
            
            let x = ( time_in_page - level_start ) - 1.2;
            let y_pos = x.powf(7.0)*x*x*x/(x.signum()+x.powf(3.0));

            static_text(tile_batcher, &mut vertices, glyph_brush, config, Vec2::new( (button_left + y_pos as f32 * 800.0) as f32, 800.0),Vec2::new( button_width, button_height ), &level_message );
        }

        if let GameState::ShowingSolution( solution_start ) = self.game_state {
            let button_width = config.width() as f32 * 0.6f32;
            let button_height = 80.0f32;
            let button_left = ( config.width() as f32 - button_width ) / 2.0;
    
            let x = ( time_in_page - solution_start ) - 1.2;
            let y_pos = x.powf(9.0)*x*x*x/(x.signum()+x*x*x);
            static_text(tile_batcher, &mut vertices, glyph_brush, config, Vec2::new( (button_left + y_pos as f32 * 800.0) as f32, 800.0),Vec2::new( button_width, button_height ), "Level Complete" );
        }

        

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let view_matrix_c: [[f32; 4]; 4] = page_manager::get_view_matrix(display);
        let uniforms = glium::uniform! {
            matrix: view_matrix_c,
//            tex: &tile_batcher.texture,
            tex: glium::uniforms::Sampler::new(&tile_batcher.texture).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).
                                                                    minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
        };

        let scissor_rect = Some( page_manager::get_scissor_rectangle(display));
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
            scissor: scissor_rect,
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };
        let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        target.draw(&vertex_buffer, &indices, &tile_batcher.program, &uniforms, &draw_params).unwrap();

        let draw_glow_params = glium::draw_parameters::DrawParameters{
            blend: glium::draw_parameters::Blend{
                color: glium::draw_parameters::BlendingFunction::Addition{
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::One
                },
                alpha: glium::draw_parameters::BlendingFunction::Addition{
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::One  //MinusSourceAlpha
                },
                constant_value: (0.0, 0.0, 0.0, 0.0 )
            },
            scissor: scissor_rect,
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };
        let glow_vertex_buffer = glium::VertexBuffer::new(display, &glow_vertices).unwrap();
        target.draw(&glow_vertex_buffer, &indices, &tile_batcher.program, &uniforms, &draw_glow_params).unwrap();
        
        glyph_brush.draw_queued_with_transform(view_matrix_c, display, &mut target);


        let mut vertices: Vec<Vertex> = Vec::new();
        let tile: u8 = Tile::Solid.into();
        let src = Vec2::new( (tile%14) as f32 / 16.0f32, 1.0-((tile/14) as f32 /16f32) );
        if let GameState::ShowingNewLevel( time_started) = self.game_state  {
            let back_depth = (1.0 - (time_in_page - time_started) as f32 * 1.7f32).max( 0.0 );
            tile_batcher.tile_color(&mut vertices, &Vec2::new( 0.0, config.height() as f32), &Vec2::new( config.width() as f32, config.height() as f32 ),
                &src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 1.0, 1.0, 1.0, back_depth ));

            if (time_in_page - time_started) > 2.5  {
                self.level.tile_movable_effect(TileEffect::SizedFadeIn( time_in_page as f32, 3.0, 1.0  ) );
                self.game_state = GameState::Playing;
            }
        }
        if let GameState::ShowingSolution( time_started) = self.game_state  {
            if time_in_page - time_started > 3.0 {
                let back_depth = ((time_in_page - time_started - 3.0 ) as f32 * 1.7f32).min( 1.0 );
                tile_batcher.tile_color(&mut vertices, &Vec2::new( 0.0, config.height() as f32), &Vec2::new( config.width() as f32, config.height() as f32 ),
                    &src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 1.0, 1.0, 1.0, back_depth ));
    
                if back_depth == 1.0 {
                    self.level_no += 1;
                    page_actions.push( PageAction::OpenLevel(self.level_no));
                    self.level = Level::load_level(self.level_no);
                    self.game_state = GameState::ShowingNewLevel( time_in_page );
                    self.level.tile_movable_effect(TileEffect::Hide );
                }
            }
        }

        if vertices.len() > 0 {
            let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
            target.draw(&vertex_buffer, &indices, &tile_batcher.program, &uniforms, &draw_params).unwrap();
        }

        target.finish().unwrap();   
    }


    fn mouse_click( &mut self, pressed: bool, pos: Vec2 ) -> PageAction{
        if self.game_state == GameState::Playing {
            if pressed {
                let map_pos = self.to_level_pos(&pos);
                if let Some( (map_x, map_y) ) =  map_pos {
                    let tile = self.level.front_tile(map_x, map_y);
                    println!( "Piece at ( {},{} ) is {:?} ( movable = {} )", map_x, map_y, tile, tile.is_movable() );
                    if tile.is_movable() {
                        self.level.set_front_tile( map_x, map_y, Tile::EmptyPiece );
                        self.tile_move = Some( TileMove{ tile, map_x, map_y, grab_cursor_pos: pos, last_cursor_pos: pos});
                    }
                }
            } else {
                if self.tile_move.is_some() {
                    let tile_move = self.tile_move.take().unwrap();
                    self.level.set_front_tile( tile_move.map_x, tile_move.map_y, tile_move.tile );
                }
            }
        } 
        return PageAction::None;
    }
    
    fn mouse_move( &mut self, new_pos: Vec2, audio: &audio::Audio ) -> PageAction{
        if self.tile_move.is_some() {
            let mut hit: u32 = 0;
            let mut moved: u32 = 0;
            let mut tile_move = self.tile_move.unwrap();
            let old_map_x = tile_move.map_x;
            let old_map_y = tile_move.map_y;
            for t in 0..15{ 
                tile_move.last_cursor_pos = new_pos.clone();
                // Should the map position change
                let mut delta = tile_move.last_cursor_pos - tile_move.grab_cursor_pos;
                if delta.x > 0.0 {
                    if tile_move.map_x == self.level.width-1 || self.level.front_tile(tile_move.map_x+1, tile_move.map_y) != Tile::EmptyPiece || 
                        self.level.back_tile(tile_move.map_x+1, tile_move.map_y) == Tile::EmptyPiece { 
                        delta.x = 0.0;
                        hit = hit | 0x01;
                    }
                } else if delta.x < 0.0 {
                    if tile_move.map_x == 0 || self.level.front_tile(tile_move.map_x-1, tile_move.map_y) != Tile::EmptyPiece ||
                        self.level.back_tile(tile_move.map_x-1, tile_move.map_y) == Tile::EmptyPiece{ 
                        delta.x = 0.0;
                        hit = hit | 0x02;
                    }
                }
                if delta.y > 0.0 {
                    if tile_move.map_y == self.level.height-1 || self.level.front_tile(tile_move.map_x, tile_move.map_y+1) != Tile::EmptyPiece ||
                        self.level.back_tile(tile_move.map_x, tile_move.map_y+1) == Tile::EmptyPiece{
                        delta.y = 0.0;
                        hit = hit | 0x04;
                    }
                } else if delta.y < 0.0 {
                    if tile_move.map_y == 0 || self.level.front_tile(tile_move.map_x, tile_move.map_y-1) != Tile::EmptyPiece || 
                        self.level.back_tile(tile_move.map_x, tile_move.map_y-1) == Tile::EmptyPiece { 
                        delta.y = 0.0;
                        hit = hit | 0x08;
                    }
                }

                if delta.x > 32.0f32 {
                    tile_move.map_x = tile_move.map_x+1;
                    delta.x = delta.x - 64.0f32;
                    tile_move.grab_cursor_pos.x = tile_move.grab_cursor_pos.x + 64.0f32;
                    moved = moved | 0x01;
                } else if delta.x < -32.0f32 {
                    tile_move.map_x = tile_move.map_x-1;
                    delta.x = delta.x + 64.0f32;
                    tile_move.grab_cursor_pos.x = tile_move.grab_cursor_pos.x - 64.0f32;
                    moved = moved | 0x02;
                }
                if delta.y > 32.0f32 {
                    tile_move.map_y = tile_move.map_y+1;
                    delta.y = delta.y - 64.0f32;
                    tile_move.grab_cursor_pos.y = tile_move.grab_cursor_pos.y + 64.0f32;
                    moved = moved | 0x04;
                } else if delta.y < -32.0f32 {
                    tile_move.map_y = tile_move.map_y-1;
                    delta.y = delta.y + 64.0f32;
                    tile_move.grab_cursor_pos.y = tile_move.grab_cursor_pos.y - 64.0f32;
                    moved = moved | 0x08;
                }
                tile_move.last_cursor_pos = tile_move.grab_cursor_pos + delta;
            }
            if self.level.front_tile(tile_move.map_x, tile_move.map_y) != Tile::EmptyPiece {
                tile_move.map_x = old_map_x;
                tile_move.map_y = old_map_y;
            }
            self.tile_move = Some( tile_move );
            if (moved & hit) != 0 {
                audio.play_sound(audio::SoundEffect::Ping);
            }
        }
        return PageAction::None;
    }
    fn key_press( &mut self, key: VirtualKeyCode, pressed: bool ) -> PageAction{ 
        if pressed {
            if key == VirtualKeyCode::Escape {
                if self.game_state == GameState::Playing {
                    self.game_state = GameState::InGameMenu;
                } else if self.game_state == GameState::InGameMenu {
                    self.game_state = GameState::Playing;
                }
            }
        }
        return PageAction::None;
    }
}