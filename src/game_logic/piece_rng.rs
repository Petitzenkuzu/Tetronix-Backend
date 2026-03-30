use crate::models::{Piece, PieceType};
use rand::{rngs::ThreadRng, Rng};

pub struct PieceRng {
    rng: ThreadRng,
    pieces: Vec<Piece>,
}

impl PieceRng {
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
            pieces: vec![],
        }
    }

    pub fn get_next_piece(&mut self) -> Piece {
        if self.pieces.is_empty() {
            self.refill();
        }
        self.pieces.pop().unwrap()
    }

    fn refill(&mut self) {
        let mut array = [0; 20];
        for i in 0..20 {
            array[i] = self.rng.random_range(0..7) as u8;
        }
        self.pieces = array
            .iter()
            .map(|&id| PieceRng::generate_piece(id))
            .collect();
    }

    fn generate_piece(id: u8) -> Piece {
        Piece::from(PieceType::from(id))
    }
}
