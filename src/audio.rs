use std::{collections::HashMap, fs::File, io::Cursor, path::PathBuf};
use std::io::Read;
use rodio::{self, OutputStream, OutputStreamHandle};
#[derive(Eq, PartialEq,Hash)]
pub enum SoundEffect{
    Ping,
    Gem,
    GemSolved,
    Transport
}
pub struct Audio{
    sounds: HashMap<SoundEffect,Vec<u8>>,
    stream: OutputStream,
    stream_handle: OutputStreamHandle
}

fn load_wav( path: PathBuf ) -> Vec<u8> {
    println!( "Loading file {:?}", path.to_str());
    let mut file = File::open( path ).unwrap();
    let mut data = Vec::new();
    file.read_to_end( &mut data).unwrap();
    return data;
}

impl Audio{
    pub fn new(path: &PathBuf) -> Audio {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        let mut sounds = HashMap::new();
        sounds.insert( SoundEffect::Ping, load_wav( path.join("sounds/click.wav") ) );
        sounds.insert( SoundEffect::Gem, load_wav( path.join("sounds/gem2.wav") ) );
        sounds.insert( SoundEffect::GemSolved, load_wav( path.join("sounds/gem.wav") ) );

        Audio{ sounds, stream, stream_handle }
    }

    pub fn play_sound( &self, effect: SoundEffect ) {
        if self.sounds.contains_key(&effect) {
            let copied_sound = self.sounds.get(&effect).unwrap().clone();
            let cursor = Cursor::new(copied_sound);
            self.stream_handle.play_once(cursor).unwrap().detach();
        }
    }
}