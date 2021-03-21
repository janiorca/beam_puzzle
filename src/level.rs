use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::{collections::HashMap, convert::TryFrom};
use std::{fs, u32};
use std::env;
use super::{Vec2};
use super::tile_batcher::*;

#[derive(Clone, Copy)]
#[derive(Debug, Eq, PartialEq, Hash, TryFromPrimitive)]
#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum Tile {
    EmptyPiece = 0,
    WallTerminatorTop = 1,
    WallHorizontal = 3,
    WallVertical = 4,
    WallTurnTopLeft = 5,
    WallTurnTopRight = 6,
    PassHorizontal = 7,
    PassVertical = 8,
    RayVertical = 9,
    RayHorizontal = 10,
    RayCross = 11,
    WallTerminatorLeft = 14,
    WallBlocker = 15,
    WallTerminatorRight = 16,
    WallTurnBottomLeft = 19,
    WallTurnBottomRight = 20,
    RayTeleport1 = 21,
    RaySourceUp = 22,
    RaySourceRight = 23,
    ImmovableTopLeft = 24,
    ImmovableTopRight = 25,
    MovableTopLeft = 26,
    MovableTopRight = 27,
    WallTerminatorBottom = 29,
    WallTLeft = 33,
    WallTRight = 34,
    RayTeleport2 = 35,
    RaySourceDown = 36,
    RaySourceLeft = 37,
    MovableBottomLeft = 40,
    MovableBottomRight = 41,
    Floor4 = 42,
    Floor5 = 43,
    Floor6 = 44,
    Floor7 = 45,
    Solid = 46,
    ImmovableBottomLeft = 38,
    ImmovableBottomRight = 39,
    WallTDown = 47,
    WallTUp = 48,
    GemRed = 49,
    GemGreen = 50,
    GemYellow = 51,
    GemPurple = 52,
    Floor1 = 53,
    Floor2 = 54,
    Floor3 =55
}

impl Default for Tile {
    fn default() -> Self {
        Self::EmptyPiece
    }
}

impl Tile{
    pub fn is_movable( &self ) -> bool {
        return ( *self == Tile::MovableBottomLeft ) || ( *self == Tile::MovableBottomRight || 
                ( *self == Tile::MovableTopLeft ) || ( *self == Tile::MovableTopRight ) );
    }

    pub fn is_ray_source( &self ) -> bool {
        return ( *self == Tile::RaySourceUp ) || ( *self == Tile::RaySourceRight || 
                ( *self == Tile::RaySourceDown ) || ( *self == Tile::RaySourceLeft ) );
    }

    pub fn is_ray_blocker( &self, direction: BeamDirection ) -> bool {
        if direction == BeamDirection::Down || direction == BeamDirection::Up {
            if *self == Tile::PassHorizontal  {
                return true;
            }
        } else {
            if *self == Tile::PassVertical  {
                return true;
            }
        }
        return *self == Tile::WallBlocker || *self == Tile::WallHorizontal || *self == Tile::WallTDown || *self == Tile::WallTerminatorBottom ||
            *self == Tile::WallTerminatorLeft || *self == Tile::WallTerminatorRight || *self == Tile::WallTerminatorTop ||
            *self == Tile::WallTLeft || *self == Tile::WallTRight || *self == Tile::WallTUp || *self == Tile::WallTurnBottomLeft ||
            *self == Tile::WallTurnBottomRight || *self == Tile::WallTurnTopLeft || *self == Tile::WallTurnTopRight || *self == Tile::WallVertical;
    }

    pub fn is_a_gem( &self ) -> bool {
        return *self == Tile::GemGreen || *self == Tile::GemRed || *self == Tile::GemYellow || *self == Tile::GemPurple;
    }

    pub fn is_teleport( &self ) -> bool {
        return *self == Tile::RayTeleport1 || *self == Tile::RayTeleport2;
    }
}

#[derive( Clone, Copy)]
pub enum TileEffect{
    None,
    Hide,
    Punch(f32,Vec2),
    SizedFadeIn(f32,f32,f32)    // start time, start_scale, duration
}

pub fn apply_tile_effect( tile_effect: &TileEffect, time_in_level: f32, pos: &Vec2, size: &Vec2, alpha: f32 ) -> ( Vec2, Vec2, f32 ) {
    match tile_effect  {
        TileEffect::None => { return (pos.clone(), size.clone(), alpha )},
        TileEffect::Hide => { return (pos.clone(), size.clone(), 0.0 )},
        TileEffect::Punch( time_started, direction) => {
            let time_in_effect = ( time_in_level - time_started )*10.0;    
            let scale = (time_in_effect*3.0).sin()*(-time_in_effect).exp();         // decayign spring
            let offset: Vec2 = direction * scale;
            return (pos + offset,  size.clone(), alpha );
        }
        TileEffect::SizedFadeIn( time_started, start_scale, duration) => {
            let time_in_effect = time_in_level - time_started;    
            let dy: f32 = 1.0 - start_scale;
            let scale_factor = start_scale + time_in_effect.min( 1.0) * dy;
            let ( scaled_pos, scaled_size ): ( Vec2, Vec2) = scale(pos,size,scale_factor );
            let scaled_alpha = time_in_effect.min( 1.0)*alpha;
            return (scaled_pos,  scaled_size, scaled_alpha );
        }
    }
}

pub fn tile_effect_to_offset( tile_effect: &TileEffect, time_in_level: f32 ) -> Vec2 {
    match tile_effect  {
        TileEffect::None => { Vec2::new( 0.0, 0.0 )},
        TileEffect::Hide => { Vec2::new( 0.0, 0.0 )},
        TileEffect::Punch( time_started, direction) => {
            let time_in_effect = ( time_in_level - time_started )*10.0;    
            let scale = (time_in_effect*3.0).sin()*(-time_in_effect).exp();
            direction * scale as f32
        }
        TileEffect::SizedFadeIn( time_started, start_scale, duration) => { Vec2::new( 0.0, 0.0 )},
    }
}
#[derive( Clone, Copy)]
struct RayTransition{
    x: u32,
    y: u32,
    time_entered: f64,
}

pub struct Level{
    pub width:u32,
    pub height: u32,
    front: Vec<u8>,
    effect:Vec<TileEffect>,
    back: Vec<u8>,
    solution: Vec<u8>,
    ray: Vec<u8>,
    
    ray_transitions: Vec<RayTransition>
}

#[derive(Debug, Eq, PartialEq)]
#[derive(Clone, Copy)]
pub enum BeamDirection{
    Up,
    Left,
    Down,
    Right
}

fn beam_direction_to_vec(direction: &BeamDirection) -> Vec2 {
    match direction {
        BeamDirection::Up => {
            Vec2::new( 0.0, 1.0)
        },
        BeamDirection::Down => {
            Vec2::new( 0.0, -1.0)
        },
        BeamDirection::Left => {
            Vec2::new( -1.0, 0.0)
        },
        BeamDirection::Right => {
            Vec2::new( 1.0, 0.0)
        },
    }
}

impl Level{
    pub fn load_level( number: u32 ) -> Level {
        let mut path_buf = env::current_dir().unwrap();
        path_buf = path_buf.join("levels/level".to_string() + &number.to_string() + &".mp".to_string());
        let input = fs::read(path_buf).unwrap();
    
    
        let width = input[1];
        let height = input[2];
        let has_solution = input[3];
        let layer_size = (width as u32 *height as u32 ) as usize;
    
        let back: Vec<u8> = input[ 4..layer_size+4].to_vec();
        let front: Vec<u8> = input[ layer_size+4..layer_size*2+4].to_vec();
        let solution: Vec<u8> = input[ (layer_size*2+4)..(layer_size*3+4)].into();
        let effect: Vec<TileEffect> = vec![TileEffect::None;width as usize*height as usize];
        return Level{ width: width as u32, height: height as u32, front, effect, back, solution, ray: vec![ Tile::EmptyPiece.into(); layer_size],
            ray_transitions: Vec::new()  };
    }

    pub fn tile_movable_effect ( &mut self, tile_effect: TileEffect ) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.front_tile( x, y ).is_movable() {
                    self.set_effect(x, y, tile_effect );
                }
            }
        }
    }

    fn find_start( &self ) -> ( u32, u32, BeamDirection ) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.front_tile( x, y ).is_ray_source() {
                    let direction = match self.front_tile(x,y) {
                        Tile::RaySourceUp => BeamDirection::Up,
                        Tile::RaySourceDown => BeamDirection::Down,
                        Tile::RaySourceLeft => BeamDirection::Left,
                        Tile::RaySourceRight => BeamDirection::Right,
                        _ => panic!( "unhandled tile direction")
                    };
                    return (x,y,direction);
                }
            }
        }
        return (0,0,BeamDirection::Up);
    }

    pub fn count_jewels( &self ) -> u32 {
        let mut gems = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.front_tile( x, y ).is_a_gem() {
                    gems += 1;
                }
            }
        }
        return gems;
    }

    fn find_teleports( &self ) -> HashMap<Tile,Vec<(u32,u32)>> {
        let mut teleport_pairs: HashMap<Tile,Vec<(u32,u32)>> = HashMap::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.front_tile(x, y);
                if tile.is_teleport() {
                    teleport_pairs.entry(tile).or_insert_with(||vec![]).push((x,y));
                }
            }
        }
        return teleport_pairs;
    }

    fn move_ray( &self, x: &mut u32, y: &mut u32, direction: &BeamDirection  ) {
        match direction {
            BeamDirection::Up => {
                if *y == 0 {
                    *y = self.height;
                }
                *y -= 1;
            },
            BeamDirection::Down => {
                *y += 1;
                if *y == self.height {
                    *y = 0;
                }
            },
            BeamDirection::Left => {
                if *x == 0 {
                    *x = self.width;
                }
                *x -= 1;
            },
            BeamDirection::Right => {
                *x += 1;
                if *x == self.width {
                    *x = 0;
                }
            },
        }
    }

    //  Return the number of jewels the ray crosses
    pub fn update_ray( &mut self, max_length: usize, time_in_page: f64) -> u32 {
        let last_ray = self.ray.clone();
        for idx in 0..self.width*self.height {
            self.ray[ idx as usize ] = 0;
        }
        let ( mut beam_x, mut beam_y, mut direction ) = self.find_start();        
        let mut jewel_count = 0;
        let teleports = self.find_teleports();
        let mut new_transitions: Vec<RayTransition> = Vec::new();

        // Move the ray out of its source ( its normally a blocker )
        self.move_ray( &mut beam_x, &mut beam_y, &direction );
        for _count in 0..max_length {
            // Check current pos
            let tile =  self.front_tile(beam_x, beam_y);
            let mut show_ray = true;
            if tile.is_ray_blocker(direction) {
                break;
            } else if tile.is_a_gem() {
                // If there is also a ray on the tile we cant count it as we have been here before
                if self.ray_tile( beam_x, beam_y ) == Tile::EmptyPiece {
                    jewel_count += 1;
                    if last_ray[ self.offset(beam_x,beam_y) ] == 0 {
                        let offset = self.offset(beam_x,beam_y);
                        self.effect[ offset ] = TileEffect::Punch( time_in_page as f32, beam_direction_to_vec(&direction)*40.0 );
                    }
                }
            }
            else if tile.is_ray_source() {
                show_ray = false;
                break;

            } else if tile == Tile::EmptyPiece {

            } else if tile == Tile::MovableTopLeft || tile == Tile::ImmovableTopLeft{
                show_ray = false;
                match direction {
                    BeamDirection::Up => { direction = BeamDirection::Right},
                    BeamDirection::Left => { direction = BeamDirection::Down},
                    _ => { break }
                }
            } else if tile == Tile::MovableTopRight || tile == Tile::ImmovableTopRight{
                show_ray = false;
                match direction {
                    BeamDirection::Right => { direction = BeamDirection::Down},
                    BeamDirection::Up => { direction = BeamDirection::Left},
                    _ => { break }
                }
            } else if tile == Tile::MovableBottomLeft || tile == Tile::ImmovableBottomLeft{
                show_ray = false;
                match direction {
                    BeamDirection::Down => { direction = BeamDirection::Right},
                    BeamDirection::Left => { direction = BeamDirection::Up},
                    _ => { break }
                }
            } else if tile == Tile::MovableBottomRight || tile == Tile::ImmovableBottomRight{
                show_ray = false;
                match direction {
                    BeamDirection::Down => { direction = BeamDirection::Left},
                    BeamDirection::Right => { direction = BeamDirection::Up},
                    _ => { break }
                }   
            }
            else if teleports.contains_key(&tile) {
                let pair = teleports.get(&tile).unwrap();
                show_ray = false;
                if beam_x == pair[ 0 ].0 && beam_y == pair[ 0 ].1 {
                    beam_x = pair[ 1 ].0;
                    beam_y = pair[ 1 ].1;
                } else {
                    beam_x = pair[ 0 ].0;
                    beam_y = pair[ 0 ].1;
                }
            }

            if show_ray {
                if self.ray_tile( beam_x, beam_y ) != Tile::EmptyPiece {
                    self.set_ray_tile(beam_x, beam_y, Tile::RayCross);
                } else {
                    match direction {
                        BeamDirection::Up | BeamDirection::Down => self.set_ray_tile(beam_x, beam_y, Tile::RayVertical),
                        BeamDirection::Left | BeamDirection::Right => self.set_ray_tile(beam_x, beam_y, Tile::RayHorizontal)
                    }
                }
            }
            // The ray briefly pauses after each turn or teleport. If we are not showing the ray, this is is one of those points
            if !show_ray { 
                let mut transition_index = None;
                for ( index, transition) in self.ray_transitions.iter().enumerate() {
                    if transition.x == beam_x && transition.y == beam_y {
                        transition_index = Some( index );
                    }
                }
                match transition_index {
                    None => {
                        // we have not encountered this before. Add it to the list
                        new_transitions.push( RayTransition{ x: beam_x, y: beam_y, time_entered: time_in_page } );
                        break;
                    },
                    Some( idx ) =>  { 
                        new_transitions.push( *self.ray_transitions.get(idx).unwrap() );
                        if ( time_in_page - new_transitions.last().unwrap().time_entered ) < 0.075 {
                            break;
                        }
                    }

                }
            }

            self.move_ray( &mut beam_x, &mut beam_y, &direction );
        }
        self.ray_transitions = new_transitions;
        return jewel_count;
    }

    fn offset( &self, x: u32, y: u32 ) -> usize {
        return (y*self.width+x) as usize;
    }

    pub fn back_tile_idx( &self, x: u32,y: u32 ) -> u8 {
        return self.back[ self.offset(x,y) ];
    }
    pub fn front_tile_idx( &self, x: u32,y: u32 ) -> u8 {
        return self.front[ self.offset(x,y) ];
    }
    pub fn ray_tile_idx( &self, x: u32,y: u32 ) -> u8 {
        return self.ray[ self.offset(x,y) ];
    }

    pub fn front_tile( &self, x: u32,y: u32 ) -> Tile {
        let tile =  Tile::try_from( self.front[ self.offset(x,y) ] ).unwrap();
        return tile;
    }

    pub fn effect( &self, x: u32,y: u32 ) -> TileEffect {
        let effect =  self.effect[ self.offset(x,y) ];
        return effect;
    }

    pub fn set_effect( &mut self, x: u32,y: u32, tile_effect: TileEffect ) {
        let offset = self.offset(x,y); 
        self.effect[ offset ] = tile_effect;
    }

    pub fn back_tile( &self, x: u32,y: u32 ) -> Tile {
        let tile =  Tile::try_from( self.back[ self.offset(x,y) ] ).unwrap();
        return tile;
    }

    pub fn ray_tile( &self, x: u32,y: u32 ) -> Tile {
        return Tile::try_from( self.ray[ self.offset(x,y) ] ).unwrap();
    }

    pub fn set_front_tile( &mut self, x: u32,y: u32, tile: Tile ) {
        let offset = self.offset(x,y); 
        self.front[ offset ] = tile.into();
    }

    pub fn set_ray_tile( &mut self, x: u32,y: u32, tile: Tile ) {
        let offset = self.offset(x,y); 
        self.ray[ offset ] = tile.into();
    }
}

