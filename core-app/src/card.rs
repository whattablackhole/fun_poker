use rand::seq::SliceRandom;
use std::collections::VecDeque;

use crate::protos::card::{Card, CardSuit, CardValue};

impl CardValue {
    pub fn from_int_to_str(i: i32) -> &'static str {
        match i {
            0 => "2",
            1 => "3",
            2 => "4",
            3 => "5",
            4 => "6",
            5 => "7",
            6 => "8",
            7 => "9",
            8 => "T",
            9 => "J",
            10 => "Q",
            11 => "K",
            12 => "A",
            _ => panic!("specify values from 0(card 2) to 12(card Ace)"),
        }
    }
}

impl CardSuit {
    pub fn from_int_to_str(val: i32) -> &'static str {
        match val {
            0 => "c",
            1 => "s",
            2 => "h",
            3 => "d",
            _ => panic!(
                "cant convert provided int to suit string, consider using values from 0 to 3"
            ),
        }
    }
}

impl Card {
    pub fn to_string(&self) -> String {
        let mut result = CardValue::from_int_to_str(self.value).to_owned();
        result.push_str(CardSuit::from_int_to_str(self.suit));
        result
    }
}

pub struct CardDeck {
    pub cards: VecDeque<Card>,
}

impl Iterator for CardValue {
    type Item = CardValue;

    fn next(&mut self) -> Option<Self::Item> {
        use CardValue::*;
        match *self {
            Two => {
                *self = Three;
                Some(Two)
            }
            Three => {
                *self = Four;
                Some(Three)
            }
            Four => {
                *self = Five;
                Some(Four)
            }
            Five => {
                *self = Six;
                Some(Five)
            }
            Six => {
                *self = Seven;
                Some(Six)
            }
            Seven => {
                *self = Eight;
                Some(Seven)
            }
            Eight => {
                *self = Nine;
                Some(Eight)
            }
            Nine => {
                *self = Ten;
                Some(Nine)
            }
            Ten => {
                *self = Jack;
                Some(Ten)
            }
            Jack => {
                *self = Queen;
                Some(Jack)
            }
            Queen => {
                *self = King;
                Some(Queen)
            }
            King => {
                *self = Ace;
                Some(King)
            }
            Ace => None,
        }
    }
}

impl Iterator for CardSuit {
    type Item = CardSuit;

    fn next(&mut self) -> Option<Self::Item> {
        use CardSuit::*;
        match *self {
            Clubs => {
                *self = Diamonds;
                Some(Clubs)
            }
            Diamonds => {
                *self = Hearts;
                Some(Diamonds)
            }
            Hearts => {
                *self = Spades;
                Some(Hearts)
            }
            Spades => None,
        }
    }
}
impl CardDeck {
    pub fn new() -> Self {
        CardDeck {
            cards: VecDeque::new(),
        }
    }

    pub fn new_random() -> CardDeck {
        let mut new_deck = VecDeque::new();
        for suit in CardSuit::Clubs {
            for value in CardValue::Two {
                new_deck.push_back(Card::new(suit, value));
            }
        }
        
        let mut rng = rand::thread_rng();
        new_deck.make_contiguous().shuffle(&mut rng);

        CardDeck { cards: new_deck }
    }
}

impl Card {
    pub fn new(s: CardSuit, v: CardValue) -> Self {
        Card {
            suit: s.into(),
            value: v.into(),
        }
    }
}
