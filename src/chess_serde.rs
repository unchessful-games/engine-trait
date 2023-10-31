pub mod position_serde {
    use std::str::FromStr;

    use serde::{
        de::{Error, Visitor},
        Deserializer, Serializer,
    };
    use shakmaty::{fen::Fen, Chess};

    pub fn serialize<S: Serializer>(b: &Chess, ser: S) -> Result<S::Ok, S::Error> {
        let fen = Fen::from_position(b.clone(), shakmaty::EnPassantMode::Legal);
        ser.serialize_str(&fen.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Chess, D::Error> {
        struct ChessVisitor {}
        impl<'de> Visitor<'de> for ChessVisitor {
            type Value = Chess;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a game state in the FEN format")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Fen::from_str(v)
                    .map_err(|_| Error::custom("error in parsing board's FEN"))?
                    .into_position(shakmaty::CastlingMode::Standard)
                    .map_err(|v| {
                        Error::custom(format!("error in parsing FEN into game position: {v}"))
                    })?)
            }
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Chess::new())
            }
        }
        d.deserialize_string(ChessVisitor {})
    }
}

pub mod uci_serde {

    use std::str::FromStr;

    use serde::{
        de::{Error, Visitor},
        Deserializer, Serializer,
    };
    use shakmaty::uci::Uci;

    pub fn serialize<S: Serializer>(u: &Uci, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&u.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Uci, D::Error> {
        struct UciVisitor {}
        impl<'de> Visitor<'de> for UciVisitor {
            type Value = Uci;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a move in the UCI format")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Uci::from_str(v).map_err(|_| Error::custom("error in parsing move's UCI"))?)
            }
        }
        d.deserialize_string(UciVisitor {})
    }
}
