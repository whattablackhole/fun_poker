use rand::seq::SliceRandom;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CardValue {
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Seven = 5,
    Eight = 6,
    Nine = 7,
    Ten = 8,
    Jack = 9,
    Queen = 10,
    King = 11,
    Ace = 12,
}
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
impl CardValue {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CardValue::Two => "Two",
            CardValue::Three => "Three",
            CardValue::Four => "Four",
            CardValue::Five => "Five",
            CardValue::Six => "Six",
            CardValue::Seven => "Seven",
            CardValue::Eight => "Eight",
            CardValue::Nine => "Nine",
            CardValue::Ten => "Ten",
            CardValue::Jack => "Jack",
            CardValue::Queen => "Queen",
            CardValue::King => "King",
            CardValue::Ace => "Ace",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Two" => Some(Self::Two),
            "Three" => Some(Self::Three),
            "Four" => Some(Self::Four),
            "Five" => Some(Self::Five),
            "Six" => Some(Self::Six),
            "Seven" => Some(Self::Seven),
            "Eight" => Some(Self::Eight),
            "Nine" => Some(Self::Nine),
            "Ten" => Some(Self::Ten),
            "Jack" => Some(Self::Jack),
            "Queen" => Some(Self::Queen),
            "King" => Some(Self::King),
            "Ace" => Some(Self::Ace),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CardSuit {
    Clubs = 0,
    Spades = 1,
    Hearts = 2,
    Diamonds = 3,
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
impl CardSuit {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CardSuit::Clubs => "Clubs",
            CardSuit::Spades => "Spades",
            CardSuit::Hearts => "Hearts",
            CardSuit::Diamonds => "Diamonds",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Clubs" => Some(Self::Clubs),
            "Spades" => Some(Self::Spades),
            "Hearts" => Some(Self::Hearts),
            "Diamonds" => Some(Self::Diamonds),
            _ => None,
        }
    }
}

// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Card {
    #[prost(enumeration = "CardValue", required, tag = "1")]
    pub value: i32,
    #[prost(enumeration = "CardSuit", required, tag = "2")]
    pub suit: i32,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardPair {
    #[prost(message, required, tag = "1")]
    pub card1: Card,
    #[prost(message, required, tag = "2")]
    pub card2: Card,
}
impl Card {
    pub fn to_string(&self) -> String {
        let mut result = CardValue::from_int_to_str(self.value).to_owned();
        result.push_str(CardSuit::from_int_to_str(self.suit));
        println!("res:{}", result);
        result
    }
}

pub struct CardDeck {
    pub deck: VecDeque<Card>,
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
    pub fn new(random: bool) -> Self {
        let mut new_deck = VecDeque::new();
        for suit in CardSuit::Clubs {
            for value in CardValue::Two {
                new_deck.push_back(Card::new(suit, value));
            }
        }
        if random == true {
            let mut rng = rand::thread_rng();
            new_deck.make_contiguous().shuffle(&mut rng);
        }
        CardDeck { deck: new_deck }
    }

    pub fn new_random() -> CardDeck {
        Self::new(true)
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
