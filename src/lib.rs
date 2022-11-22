pub mod api;
pub mod entity;

pub mod deserializer {
    use chrono::{DateTime, Utc};
    use core::fmt;
    use serde::de;

    pub(super) struct TimeStampVisitor;

    impl<'de> de::Visitor<'de> for TimeStampVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a UTC or custom UTC(2015-07-08T02:50:59.97)")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            use std::str::FromStr;
            match DateTime::<Utc>::from_str(value) {
                Ok(datetime) => Ok(datetime),
                Err(_) => {
                    let value = format!("{value}+00:00");
                    DateTime::<Utc>::from_str(&value).map_err(de::Error::custom)
                }
            }
        }
    }

    pub mod timestamp {
        use super::TimeStampVisitor;
        use chrono::{DateTime, Utc};
        use serde::de;

        pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_str(TimeStampVisitor)
        }
    }

    pub mod timestamp_option {
        use chrono::{DateTime, Utc};
        use serde::de;

        pub fn deserialize<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            use serde::Deserialize;
            #[derive(Deserialize)]
            struct Helper(#[serde(with = "super::timestamp")] DateTime<Utc>);
            let helper = Option::deserialize(d)?;
            Ok(helper.map(|Helper(x)| x))
        }
    }
}
