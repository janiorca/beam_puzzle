use std::convert::TryInto;
use super::Vec2;
use super::Vec4;
use super::Vertex;

pub struct TileBatcher{
    pub program: glium::Program,
    pub texture: glium::texture::srgb_texture2d::SrgbTexture2d,
}

impl TileBatcher{
    pub fn new( texture: glium::texture::srgb_texture2d::SrgbTexture2d, display: &glium::Display, logical_size: Vec2 ) -> TileBatcher {
        let vertex_shader_src = r#"
            #version 140
            in vec2 position;
            in vec2 tex_coords;
            in vec4 color;
            out vec2 v_tex_coords;
            out vec4 v_color;
            
            uniform mat4 matrix;
            
            void main() {
                v_tex_coords = tex_coords;
                v_color = color;
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            in vec2 v_tex_coords;
            in vec4 v_color;
            out vec4 color;
            
            uniform sampler2D tex;
            
            void main() {
                color = texture(tex, v_tex_coords)*v_color;
            }
        "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();
        let glow_program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        TileBatcher{ program, texture }
    }
    
    pub fn tile( &self, vertices: &mut Vec<Vertex>, dest: &Vec2, dest_size: &Vec2, src: &Vec2, src_size: &Vec2 ) {
        self.tile_color( vertices, dest, dest_size, src, src_size, &Vec4::new( 1.0, 1.0, 1.0, 1.0 ) );
    } 

    pub fn tile_color( &self, vertices: &mut Vec<Vertex>, dest: &Vec2, dest_size: &Vec2, src: &Vec2, src_size: &Vec2, tile_color: &Vec4 ) {
        let color: [ f32; 4 ] = tile_color.as_slice().try_into().unwrap();
        let mut dest_pos = dest.clone();
        let mut src_pos = src.clone();
        vertices.push( Vertex{ position: dest_pos.into(), tex_coords: src_pos.into(), color: color.clone()} );
        dest_pos = dest_pos + Vec2::new( dest_size.x, 0.0 );
        src_pos = src_pos + Vec2::new( src_size.x, 0.0 );
        vertices.push( Vertex{ position: dest_pos.into(), tex_coords: src_pos.into(), color: color.clone()} );
        dest_pos = dest_pos + Vec2::new( 0.0, -dest_size.y );
        src_pos = src_pos + Vec2::new( 0.0, -src_size.y );
        vertices.push( Vertex{ position: dest_pos.into(), tex_coords: src_pos.into(), color: color.clone()} );

        vertices.push( Vertex{ position: dest_pos.into(), tex_coords: src_pos.into(), color: color.clone()} );
        dest_pos = dest_pos + Vec2::new( -dest_size.x, 0.0 );
        src_pos = src_pos + Vec2::new( -src_size.x, 0.0 );
        vertices.push( Vertex{ position: dest_pos.into(), tex_coords: src_pos.into(), color: color.clone()} );
        dest_pos = dest_pos + Vec2::new( 0.0, dest_size.y );
        src_pos = src_pos + Vec2::new( 0.0, src_size.y );
        vertices.push( Vertex{ position: dest_pos.into(), tex_coords: src_pos.into(), color: color.clone()} );
    } 

}

pub fn scale( pos: &Vec2, size: &Vec2, scale: f32 ) -> ( Vec2, Vec2 ) {
    let new_size = scale*size;
    let new_pos = Vec2::new( pos.x - ( new_size.x - size.x ) / 2f32, pos.y + ( new_size.y - size.y ) / 2f32);
    return ( new_pos, new_size);
}