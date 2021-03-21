use crate::{Vertex, level::Tile};
use glium_glyph::{GlyphBrush, glyph_brush::Section};

use super::{Vec2,Vec4};
use super::tile_batcher::TileBatcher;
use super::config::Config;
use super::page_manager::PageAction;

#[derive(Debug, Eq, PartialEq)]
pub enum ButtonState{
    Up,     
    PressedDown,
    Down,
    ReleasedUp
}

pub struct MouseState{
    pub pos: Vec2,
    pub button_state: ButtonState
}

pub fn static_text( tile_batcher: &TileBatcher, vertices: &mut Vec<Vertex>, glyph_brush: &mut GlyphBrush, config: &Config, pos: Vec2, size: Vec2, text: &str ){

    let tile: u8 = Tile::Solid.into();
    let src = Vec2::new( (tile%14) as f32 / 16.0f32, 1.0-((tile/14) as f32 /16f32) );

    let back_depth = 0.7f32;
    tile_batcher.tile_color(vertices, &pos, &size,&src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 0.0, 0.0, 0.0, back_depth ));

    glyph_brush.queue(Section {
        text: text,
        layout: glyph_brush::Layout::default_single_line().h_align(glyph_brush::HorizontalAlign::Center).v_align(glyph_brush::VerticalAlign::Center),
        screen_position: (pos.x + size.x/2.0, config.height() as f32 - pos.y + size.y/2.0f32),
        scale: glyph_brush::rusttype::Scale::uniform(48.0),
        color: [ 1.0, 1.0, 1.0, 0.8 ],
        ..Section::default()
    });
}

pub fn button( tile_batcher: &TileBatcher, vertices: &mut Vec<Vertex>, glyph_brush: &mut GlyphBrush, config: &Config, pos: Vec2, size: Vec2, text: &str, latest_mouse: &MouseState,
    click_closure: &mut FnMut() ){
    let mouse_pos = Vec2::new( latest_mouse.pos.x, config.height() as f32- latest_mouse.pos.y );
    let mut hover = false;
    if mouse_pos.x > pos.x && mouse_pos.x < pos.x + size.x && mouse_pos.y < pos.y && mouse_pos.y > pos.y - size.y {
        hover = true;
    }

    let tile: u8 = Tile::Solid.into();
    let src = Vec2::new( (tile%14) as f32 / 16.0f32, 1.0-((tile/14) as f32 /16f32) );

    let back_depth = if hover { 0.7f32 } else { 0.5 };
    tile_batcher.tile_color(vertices, &pos, &size,&src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 0.0, 0.0, 0.0, back_depth ));

    glyph_brush.queue(Section {
        text: text,
        layout: glyph_brush::Layout::default_single_line().h_align(glyph_brush::HorizontalAlign::Center).v_align(glyph_brush::VerticalAlign::Center),
        screen_position: (pos.x + size.x/2.0, config.height() as f32 - pos.y + size.y/2.0f32),
        scale: glyph_brush::rusttype::Scale::uniform(48.0),
        color: if hover { [ 1.0, 1.0, 1.0, 0.8 ] } else { [ 1.0, 1.0, 1.0, 0.4 ] },
        ..Section::default()
    });

    if hover && latest_mouse.button_state == ButtonState::PressedDown {
        click_closure();
    }
}

pub fn multi_selector( tile_batcher: &TileBatcher, vertices: &mut Vec<Vertex>, glyph_brush: &mut GlyphBrush, config: &Config, pos: Vec2, size: Vec2, latest_mouse: &MouseState,
        title: &str, options: &Vec<&str>, selection: usize, click_closure: &mut FnMut(usize) ) {

    let option_width = size.x / options.len() as f32;
    let option_height = size.y / 2.0;
    let mouse_pos = Vec2::new( latest_mouse.pos.x, config.height() as f32- latest_mouse.pos.y );

    let mut hover_idx: Option<usize> = None;

    if mouse_pos.x > pos.x && mouse_pos.x < pos.x + size.x && mouse_pos.y < pos.y - option_height && mouse_pos.y > pos.y - size.y {
        let idx = (( mouse_pos.x - pos.x ) as usize / option_width as usize).min( options.len() );
        hover_idx = Some(idx);
    }

    let tile: u8 = Tile::Solid.into();
    let src = Vec2::new( (tile%14) as f32 / 16.0f32, 1.0-((tile/14) as f32 /16f32) );

    // draw everyting
    tile_batcher.tile_color(vertices, &pos, &size,&src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 0.0, 0.0, 0.0, 0.4 ));
    // darken selection
    let sel_dest = Vec2::new( selection as f32 * option_width+pos.x, pos.y - option_height );
    let sel_size = Vec2::new( option_width, size.y/2.0f32 );
    tile_batcher.tile_color(vertices, &sel_dest, &sel_size,&src, &Vec2::new( 1.0/16.0, 1.0/16.0 ), &Vec4::new( 0.0, 0.0, 0.0, 0.5 ));


    glyph_brush.queue(Section {
        text: title,
        layout: glyph_brush::Layout::default_single_line().h_align(glyph_brush::HorizontalAlign::Center).v_align(glyph_brush::VerticalAlign::Center ),
        screen_position: (pos.x + size.x/2.0f32, config.height() as f32 - pos.y + option_height / 2.0f32 ),
        scale: glyph_brush::rusttype::Scale::uniform(48.0),
        color: [ 1.0, 1.0, 1.0, 0.6 ],
        ..Section::default()
    });
    for ( idx, str_item) in options.iter().enumerate() {

        glyph_brush.queue(Section {
            text: str_item,
            layout: glyph_brush::Layout::default_single_line().h_align(glyph_brush::HorizontalAlign::Center).v_align(glyph_brush::VerticalAlign::Center ),
            screen_position: (pos.x + option_width*idx as f32 + option_width/2f32, config.height() as f32 - pos.y  + option_height * 3.0f32 / 2.0f32 ),
            scale: glyph_brush::rusttype::Scale::uniform(48.0),
            color: if hover_idx.is_some() && hover_idx.to_owned().unwrap() == idx { [ 1.0, 1.0, 1.0, 0.8 ] } else { [ 1.0, 1.0, 1.0, 0.4 ] },
            ..Section::default()
        });
    }
    if hover_idx.is_some() && latest_mouse.button_state == ButtonState::PressedDown {
        click_closure(hover_idx.unwrap());
    }
}
