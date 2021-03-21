use crate::level::apply_tile_effect;

use super::Level;
use super::tile_batcher::*;
use super::{Vertex, level::Tile, level::tile_effect_to_offset};
use super::{Vec2,Vec4};
use super::config::Config;
use super::glm;
use std::time::SystemTime;

pub fn render_level(config: &Config, tile_batcher: &TileBatcher,  level: &Level, vertices: &mut Vec<Vertex>, time_in_level: f64 ) {
    let qtr_pixel=1.02f32/( 16f32 * 64f32 * 4f32 );
    let half_pixel= 1.02f32/( 16f32 * 64f32 * 2f32 );

    let ms_offset = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() % 10000;
    let s_offset = ms_offset as f32/ 10000.0f32;
    let offset = Vec2::new( 48.0*s_offset ,48.0*s_offset+32.0 );
    for y in 0..22{
        for x in -2..16 {
            let tile: u8 = Tile::Floor2.into();
            let src = Vec2::new( (tile%14) as f32 / 16.0f32+qtr_pixel, 1.0-((tile/14) as f32 /16f32)-qtr_pixel );
            tile_batcher.tile_color(vertices, &(Vec2::new( x as f32*48.0, config.height() as f32 - y as f32*48.0 ) + offset), &Vec2::new( 48.0, 48.0 ), 
                &src, &Vec2::new( 1.0/16.0-half_pixel, 1.0/16.0-half_pixel ), &Vec4::new( 0.1f32, 0.1, 0.15, 1.0 ));
        }
    }

    for y in 0..level.height{
        for x in 0..level.width {
            let tile = level.back_tile_idx(x,y);
            let src = Vec2::new( (tile%14) as f32 / 16.0f32+qtr_pixel, 1.0-((tile/14) as f32 /16f32)-qtr_pixel );
            tile_batcher.tile(vertices,&Vec2::new( x as f32*64.0, config.height() as f32 - y as f32*64.0 ), &Vec2::new( 64.0, 64.0 ), &src, &Vec2::new( 1.0/16.0-half_pixel, 1.0/16.0-half_pixel ));
        }
    }

    for y in 0..level.height{
        for x in 0..level.width {
            let tile = level.front_tile_idx(x,y);
            let src: Vec2 = Vec2::new( (tile%14) as f32 / 16.0f32+qtr_pixel, 1.0-((tile/14) as f32 /16f32)-qtr_pixel );
            let dest: Vec2 = Vec2::new( x as f32*64.0, config.height() as f32 - y as f32*64.0 );
            let (final_pos, final_size, final_alpha ) : (Vec2,Vec2,f32 )=  apply_tile_effect(&level.effect(x,y), time_in_level as f32, &dest, &Vec2::new( 64.0, 64.0 ), 1.0f32 );
            tile_batcher.tile_color(vertices,&final_pos, &final_size, &src, &Vec2::new( 1.0/16.0-half_pixel, 1.0/16.0-half_pixel ), &Vec4::new( 1f32, 1f32, 1f32, final_alpha ));
        }
    }
}

pub fn render_level_glows(config: &Config, tile_batcher: &TileBatcher,  level: &Level, vertices: &mut Vec<Vertex>, time_in_level: f64 ) {
    let qtr_pixel=1.02f32/( 16f32 * 64f32 * 4f32 );
    let half_pixel= 1.02f32/( 16f32 * 64f32 * 2f32 );

    let ms_offset = 1f32 + (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()*200f64).sin() as f32 * 0.05f32;
    for y in 0..level.height{
        for x in 0..level.width {
            if level.front_tile(x,y).is_a_gem() {
                let tile = level.front_tile_idx(x,y);
                let src = Vec2::new( (tile%14) as f32 / 16.0f32+qtr_pixel, 1.0-((tile/14) as f32 /16f32)-qtr_pixel );

                let strength = if level.ray_tile_idx(x,y) == 0 { 4 } else { 12 };
                let mut scale_factor= ms_offset;
                let offset: Vec2 = tile_effect_to_offset( &level.effect(x,y), time_in_level as f32 );
                let dest: Vec2 = Vec2::new( x as f32*64.0, config.height() as f32 - y as f32*64.0 ) + offset;
                scale_factor += glm::length( &offset )/30.0;

                for g in 0..strength {
                    let ( scaled_pos, scaled_size ) = 
                        scale(&dest, &Vec2::new( 64.0, 64.0 ),scale_factor );
                    tile_batcher.tile_color(vertices,&scaled_pos, &scaled_size, &src, &Vec2::new( 1.0/16.0-half_pixel, 1.0/16.0-half_pixel ), &Vec4::new( 1.0, 1.0, 1.0, 0.1f32 ));
                    scale_factor *= 1.05;
                }
            }
        }
    }


    let ms_offset = (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()*130f64).sin() as f32 * 0.05f32;
    
    for y in 0..level.height{
        for x in 0..level.width {
            let tile = level.ray_tile_idx(x,y);
            if tile == 0 {
                continue;
            }
            let mut scale_factor = ms_offset;
            for _g in 0..1 {

                let src = Vec2::new( (tile%14) as f32 / 16.0f32+qtr_pixel, 1.0-((tile/14) as f32 /16f32)-qtr_pixel );
                let ( scaled_pos, scaled_size ) = 
                    scale(&Vec2::new( x as f32*64.0, config.height() as f32 - y as f32*64.0 ), &Vec2::new( 64.0, 64.0 ), 1f32 );// scale_factor);

                tile_batcher.tile_color(vertices,&scaled_pos, &scaled_size, &src, &Vec2::new( 1.0/16.0-half_pixel, 1.0/16.0-half_pixel ), 
                     &Vec4::new( 1.0, 1.0, 1.0, 0.5f32+scale_factor ));

                scale_factor *= 1.05;
    
            }
        }
    }
 
}