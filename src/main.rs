#[macro_use]
extern crate glium;
mod game;
mod main_menu;
mod settings;
mod tile_batcher;
mod page;
mod page_manager;
mod level;
mod ui;
mod render_level;
mod audio;
mod config;
mod editable_constants;

use glium_glyph::glyph_brush::{rusttype::Font};
use glium_glyph::GlyphBrush;
use glium_glyph::GlyphBrushBuilder;


use level::Level as Level;
use glium::{glutin};
use glutin::{event::{ElementState, MouseButton}, platform::{windows::WindowBuilderExtWindows}};
use tile_batcher::TileBatcher;
use std::{collections::HashMap, env};
use nalgebra_glm as glm;
use nalgebra_glm::Vec2 as Vec2;
use nalgebra_glm::Vec3 as Vec3;
use nalgebra_glm::Vec4 as Vec4;
use nalgebra_glm::Mat4 as Mat4;

use crate::page::Page;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32;4]
}

glium::implement_vertex!(Vertex, position, tex_coords,color);        // don't forget to add `tex_coords` here


fn main() {
    let config = config::Config::new( );

    let event_loop = glutin::event_loop::EventLoop::new();
    // Need to disable drag and drop because it would enable a windows thread apartment model that is incompatible with the thread aparment model used by cpal ( which is used by rodio )
    let mut wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::LogicalSize::new(config.width(), config.height())).with_drag_and_drop(false);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24).with_srgb(false).with_pixel_format(8, 8).with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let audio = audio::Audio::new(&env::current_dir().unwrap());


//    let image = image::load(Cursor::new(&include_bytes!("assets/walls2.png")[..]), image::ImageFormat::Png).unwrap().to_rgba8();
//    let image = image::load(Cursor::new(&include_bytes!("assets/opengl.png")[..]), image::ImageFormat::Png).unwrap().to_rgba8();

    let mut path_buf = env::current_dir().unwrap();
    path_buf = path_buf.join("walls2.png");

    let decoder = png::Decoder::new(std::fs::File::open(path_buf.as_path()).unwrap());
    let (col_info, mut reader) = decoder.read_info().unwrap();
    println!( "Texture image: {:?}", col_info);
    let mut tex_map = vec![0; col_info.buffer_size()];
    reader.next_frame(&mut tex_map).unwrap();

    
//    let image_dimensions2 = image.dimensions();
//    let image2 = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions2);
    let image_dimensions = (col_info.width, col_info.height);
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(tex_map.as_slice(), image_dimensions);
//    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

//    let texture = glium::texture::srgb_texture2d::SrgbTexture2d::new( &display, image).unwrap();
    let texture = glium::texture::srgb_texture2d::SrgbTexture2d::new( &display, image).unwrap();

    let game_page = Box::new( game::GamePage::new(&config));
    let main_menu_page = Box::new( main_menu::MainMenuPage::new());
    let settings_page = Box::new( settings::SettingsPage::new());

    let mut pages: HashMap<page_manager::PageName, Box<dyn Page>> = HashMap::new();
    pages.insert(page_manager::PageName::Game, game_page);
    pages.insert(page_manager::PageName::MainMenu, main_menu_page);
    pages.insert(page_manager::PageName::Settings, settings_page);
    let mut page_manager = page_manager::PageManager::new( pages, page_manager::PageName::MainMenu, audio, config );
//    let mut page_manager = page_manager::PageManager::new( pages, page_manager::PageName::MainMenu, audio, config, glyph_brush );

    
    let mut last_mouse_pos = Vec2::new(0.0,0.0);
    let mut tile_batcher = TileBatcher::new( texture, &display, Vec2::new( page_manager::LOGICAL_WIDTH as f32, page_manager::LOGICAL_HEIGHT as f32 ));




    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::CursorMoved{ device_id: _, position, modifiers: _ } => {
//                    println!( "Moved to {}, {}", position.x, position.y );
                    last_mouse_pos.x = position.x as f32;
                    last_mouse_pos.y = position.y as f32;
                    page_manager.mouse_move(last_mouse_pos, &display);
                },
                glutin::event::WindowEvent::MouseInput{ device_id: _, state, button, modifiers: _ } => {
                    if button == MouseButton::Left {
                        page_manager.mouse_click(state == ElementState::Pressed, last_mouse_pos.clone(),&display);
                    }
                },
                glutin::event::WindowEvent::KeyboardInput{ device_id: _, input, is_synthetic } =>{
                    if let Some( key_code) = input.virtual_keycode {
                        page_manager.key_press( key_code, input.state == ElementState::Pressed,&display );
                    }
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            glutin::event::Event::MainEventsCleared => {
                page_manager.tick( &display, &tile_batcher );
            },            

            _ => return,
        }
        if page_manager.should_exit() {
            *control_flow = glutin::event_loop::ControlFlow::Exit;
            return;
        }
    });
}
