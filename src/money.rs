// src/money.rs
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Add;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Currency {
    VAC, // Virtual auction currency
    SEK, // Swedish Krona
    DKK, // Danish Krone
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::VAC => write!(f, "VAC"),
            Currency::SEK => write!(f, "SEK"),
            Currency::DKK => write!(f, "DKK"),
        }
    }
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VAC" => Ok(Currency::VAC),
            "SEK" => Ok(Currency::SEK),
            "DKK" => Ok(Currency::DKK),
            _ => Err(format!("Unknown currency: {}", s)),
        }
    }
}

pub type AmountValue = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount {
    currency: Currency,
    value: AmountValue,
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Amount::from_str(&text)
            .map_err(serde::de::Error::custom)
    }
}

impl Amount {
    pub fn new(currency: Currency, value: i64) -> Self {
        Amount { currency, value }
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, Error)]
pub enum MoneyError {
    #[error("Cannot add amounts with different currencies")]
    CurrencyMismatch,
}

impl Add for Amount {
    type Output = Result<Amount, MoneyError>;

    fn add(self, other: Self) -> Self::Output {
        if self.currency == other.currency {
            Ok(Amount {
                currency: self.currency,
                value: self.value + other.value,
            })
        } else {
            Err(MoneyError::CurrencyMismatch)
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.currency, self.value)
    }
}

impl FromStr for Amount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let currency_end = s.chars().take_while(|c| c.is_alphabetic()).count();
        if currency_end == 0 {
            return Err("Invalid amount format: no currency".to_string());
        }

        let currency_str = &s[..currency_end];
        let currency = Currency::from_str(currency_str)?;

        let value_str = &s[currency_end..];
        let value = value_str.parse::<i64>()
            .map_err(|_| format!("Invalid amount value: {}", value_str))?;

        Ok(Amount { currency, value })
    }
} 