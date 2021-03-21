use std::collections::HashMap;
//use glium_glyph::GlyphBrush;
use glium_glyph::glyph_brush::{rusttype::Font};

//use super::glyph_brush::{rusttype::Font};
use super::GlyphBrush;
use super::GlyphBrushBuilder;

use super::glutin::event::VirtualKeyCode;
use super::{config, tile_batcher::TileBatcher};
use super::Page;
use super::{Vec2,Vec3, Vec4, Mat4};
use super::ui::{MouseState, ButtonState};
use super::glium::{glutin};
use super::glm;
use super::audio;
use std::{time::SystemTime};

pub static LOGICAL_WIDTH: u32 = 64*11;
pub static LOGICAL_HEIGHT: u32 = 64*15;

#[derive(Debug, Eq, PartialEq, Hash)]
#[derive(Clone, Copy)]
pub enum PageName{
    MainMenu,
    Settings,
    Game
}

#[derive(Debug, Eq, PartialEq, Hash)]
#[derive(Clone, Copy)]
pub enum PageAction{
    None,
    Exit,
    VisitPage( PageName ),          // visits a page. use Back to go to previous
    Back,                           // Return to previous page ( Entered from using VisitPage )
    
    SetFullScreen( bool ),
    OpenLevel( u32 ),
}

struct PageStackEntry{
    page: PageName,
    time_effective_entered_page: f64,         
    time_page_stacked: f64,
}   

pub struct GlyphBrushInstance<'a>{
    glyph_brush: Option<GlyphBrush<'a,'a>>,
    screen_width: u32,
    screen_height: u32,
}
impl <'a>GlyphBrushInstance<'a>{
    fn new( ) -> GlyphBrushInstance<'a> {
        GlyphBrushInstance{ glyph_brush: None, screen_width: 0, screen_height:0 }
    }

    fn get_glyph_brush(&mut self, display: &glium::Display) -> &mut GlyphBrush<'a,'a>{
        let inner_size = display.gl_window().window().inner_size();

        if inner_size.width != self.screen_width || inner_size.height != self.screen_height  {
            println!( "Screen resolution changed. Dropping old glyph_brush");
            self.glyph_brush = None;
        }
        if self.glyph_brush.is_none() {
            println!( "Creating glyph_brush");
            let dejavu: &[u8] = include_bytes!("../fonts/DejaVuSans-2.37.ttf");
            let fonts = vec![Font::from_bytes(dejavu).unwrap()];
            let glyph_brush_builder = GlyphBrushBuilder::using_fonts( fonts);

            
            let scissor = get_scissor_rectangle(display);
            
            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: Some( scissor ),
                ..Default::default()
            };
            self.glyph_brush = Some(glyph_brush_builder.params(params).build(display));


            self.screen_width = inner_size.width;
            self.screen_height = inner_size.height;
        }
        return self.glyph_brush.as_mut().unwrap();
    }
}



pub struct PageManager<'a>{
    pages:  HashMap<PageName, Box<dyn Page>>,
    current_page: PageName,
    should_exit: bool,

    page_stack: Vec<PageStackEntry>,
    mouse_state: MouseState,

    audio: audio::Audio,
    time_effective_entered_page: f64,           // Effective time of when current page was entered. Not always the actual time as time is frozen when the page is pushed

    config: config::Config,
    
    glyph_brush_instance: GlyphBrushInstance<'a>
}

impl <'a> PageManager<'a>{
    fn handle_page_action( &mut self, page_action: PageAction, display: &glium::Display ) {
        match page_action{
            PageAction::None => return,
            PageAction::Exit => {
                self.should_exit = true;
            },
            PageAction::VisitPage( new_page ) => {
                println!( "Visiting page {:?}", new_page );
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
                self.page_stack.push(PageStackEntry{ page: self.current_page, time_effective_entered_page: self.time_effective_entered_page, time_page_stacked: now });
                println!( "Effective time in current page {}", self.time_effective_entered_page-now);

                self.current_page = new_page;
                self.pages.get_mut( &self.current_page ).unwrap().enter();
                self.time_effective_entered_page = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
            },
            PageAction::Back => {
                println!( "Back from page" );
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
                let page_entry = self.page_stack.pop().unwrap();
                self.current_page = page_entry.page;
                println!( "Current page is now {:?},", self.current_page );

                self.time_effective_entered_page = page_entry.time_effective_entered_page + ( now - page_entry.time_page_stacked );
                println!( "Effective time in current page {}", self.time_effective_entered_page - now);
            }
            PageAction::SetFullScreen( full_screen ) => {
                if full_screen {
                    // Are we already full screen
                    if display.gl_window().window().fullscreen().is_none() {
                        let monitor_handle = display.gl_window().window().current_monitor().unwrap();
                        let fs = glutin::window::Fullscreen::Borderless(Some(monitor_handle));
                        display.gl_window().window().set_fullscreen(Some(fs));
                        self.config.set_full_screen( true );
                    } 
                } else {
                    if display.gl_window().window().fullscreen().is_some() {
                        display.gl_window().window().set_fullscreen(None);
                        self.config.set_full_screen( false );
                    }
                }
            }

            PageAction::OpenLevel( level_opened ) => {
                self.config.increase_max_level( level_opened );
            }
        }
    }
}

impl <'a>PageManager<'a>{
//    pub fn new( pages: HashMap<PageName, Box<dyn Page>>, start_page: PageName, audio: audio::Audio, config: config::Config, glyph_brush: glium_glyph::GlyphBrush<'a, 'a>) -> PageManager<'a> {
    pub fn new( pages: HashMap<PageName, Box<dyn Page>>, start_page: PageName, audio: audio::Audio, config: config::Config ) -> PageManager<'a> {
        let time_effective_entered_page = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();

        return PageManager{ pages, current_page: start_page, should_exit: false, page_stack: Vec::new(), 
                mouse_state: MouseState{ pos: Vec2::new( 0.0, 0.0 ), button_state: ButtonState::Up }, audio, 
                time_effective_entered_page, config, glyph_brush_instance: GlyphBrushInstance::new() };
    }

    pub fn should_exit( &self ) -> bool {
        return self.should_exit;
    }

    pub fn mouse_move( &mut self, pos: Vec2, display: &glium::Display ) {
        let logical_pos =  get_logical_pos( display, &pos );
        let action = self.pages.get_mut( &self.current_page ).unwrap().mouse_move( logical_pos.clone_owned(), &self.audio );
        self.mouse_state.pos = logical_pos;
        self.handle_page_action( action, display );
    }

    pub fn mouse_click( &mut self, pressed: bool, pos: Vec2, display: &glium::Display ) {
        let logical_pos =  get_logical_pos( display, &pos );
        let action = self.pages.get_mut( &self.current_page ).unwrap().mouse_click( pressed, logical_pos.clone_owned() );
        self.mouse_state.button_state = if pressed { ButtonState::PressedDown } else { ButtonState::ReleasedUp };
        self.mouse_state.pos = logical_pos;
        self.handle_page_action( action, display );
    }

    pub fn tick( &mut self, display: &glium::Display, tile_batcher: &TileBatcher ) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
        let time_in_page = now - self.time_effective_entered_page;

        let mut page_actions = Vec::new();

        let glyph_brush = self.glyph_brush_instance.get_glyph_brush(display);
        self.pages.get_mut( &self.current_page ).unwrap().tick( display, &self.config, tile_batcher, glyph_brush, &self.mouse_state, &self.audio, time_in_page, &mut page_actions );
        if page_actions.len() > 0 {
            println!( "Doing actions {} ", page_actions.len() );
            for action in page_actions {
                println!( "Doing action {:?} ", action );
                self.handle_page_action( action, display );
            }
        }
            
        if self.mouse_state.button_state == ButtonState::PressedDown {
            self.mouse_state.button_state = ButtonState::Down;
        }
        if self.mouse_state.button_state == ButtonState::ReleasedUp {
            self.mouse_state.button_state = ButtonState::Up;
        }
    }

    pub fn key_press( &mut self, key: VirtualKeyCode, pressed: bool, display: &glium::Display ) {
        let action = self.pages.get_mut( &self.current_page ).unwrap().key_press( key, pressed );
        self.handle_page_action( action, display );
    }
}

pub fn get_logical_pos(display: &glium::Display, screen_pos: &Vec2 ) -> Vec2 {
    // Simplify this madness
    let inner_size = display.gl_window().window().inner_size();
    let scale_x =inner_size.width as f32 /  LOGICAL_WIDTH as f32;
    let scale_y =inner_size.height as f32 /  LOGICAL_HEIGHT as f32;
    if scale_x > scale_y {
        let lx = screen_pos.x / scale_y - ( inner_size.width as f32 /scale_y - LOGICAL_WIDTH as f32 )  / 2f32;
        let ly = screen_pos.y / scale_y;
        return Vec2::new( lx, ly );
    } else {
        let lx = screen_pos.x / scale_x;
        let ly = screen_pos.y / scale_x - ( inner_size.height as f32 /scale_x - LOGICAL_HEIGHT as f32 )  / 2f32;
        return Vec2::new( lx, ly );
    }
}

pub fn get_screen_pos(display: &glium::Display, logical_pos: &Vec2 ) -> Vec2 {
    let inner_size = display.gl_window().window().inner_size();
    let scale_x =inner_size.width as f32 /  LOGICAL_WIDTH as f32;
    let scale_y =inner_size.height as f32 /  LOGICAL_HEIGHT as f32;
    if scale_x > scale_y {
        let sx = ( logical_pos.x + ( inner_size.width as f32 /scale_y - LOGICAL_WIDTH as f32 )  / 2f32 ) * scale_y;
        let sy = logical_pos.y * scale_y;
        return Vec2::new( sx, sy );
    } else {
        let sx = logical_pos.y * scale_x;
        let sy = ( logical_pos.y + ( inner_size.height as f32 /scale_x - LOGICAL_HEIGHT as f32 )  / 2f32 ) * scale_x;
        return Vec2::new( sx, sy );
    }
}

pub fn get_scissor_rectangle( display: &glium::Display ) -> glium::Rect {
    let bl = get_screen_pos(display, &Vec2::new( 0.0, 0.0));
    let tr = get_screen_pos(display, &Vec2::new( LOGICAL_WIDTH as f32, LOGICAL_HEIGHT as f32));
    glium::Rect{ left: bl.x as u32, bottom: bl.y as u32, width: (tr.x - bl.x) as u32, height: (tr.y - bl.y) as u32 }
}

pub fn get_view_matrix(display: &glium::Display ) -> [[f32; 4]; 4] {
    let inner_size = display.gl_window().window().inner_size();
    let scale_x =inner_size.width as f32 /  LOGICAL_WIDTH as f32;
    let scale_y =inner_size.height as f32 /  LOGICAL_HEIGHT as f32;
    let view_matrix: Mat4;
    if scale_x > scale_y {
        let offset = ( inner_size.width as f32 / scale_y - LOGICAL_WIDTH as f32 ) / 2f32;
        let left_offset = offset.floor();
        let right_offset = offset + ( offset - left_offset );

        view_matrix = glm::ortho(-left_offset, LOGICAL_WIDTH as f32 + right_offset, 0.0, LOGICAL_HEIGHT as f32, -1.0, 1.0);
    } else {
        let offset = ( inner_size.height as f32 / scale_x - LOGICAL_HEIGHT as f32 ) / 2f32;
        let bottom_offset = offset.floor();
        let top_offset = offset + ( offset - bottom_offset );
        view_matrix = glm::ortho(0f32, LOGICAL_WIDTH as f32, -bottom_offset, LOGICAL_HEIGHT as f32+top_offset, -1.0, 1.0);
    }

    let view_matrix_c: [[f32; 4]; 4] = view_matrix.into();
    return view_matrix_c;
}
