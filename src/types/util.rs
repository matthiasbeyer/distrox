use std::ops::Deref;

use chrono::NaiveDateTime;
use mime::Mime;

/// A simple version.
///
/// Like "1" for example
#[derive(Serialize, Deserialize, Debug)]
pub struct Version(usize);

impl From<usize> for Version {
    fn from(u: usize) -> Self {
        Version(u)
    }
}

/// A Timestamp which can be parsed into a NaiveDateTime object
/// Format: 2014-11-28T21:45:59.324310806+09:00
/// (RFC3999)
#[derive(Serialize, Deserialize, Debug)]
pub struct Timestamp(NaiveDateTime);

impl From<NaiveDateTime> for Timestamp {
    fn from(ndt: NaiveDateTime) -> Self {
        Timestamp(ndt)
    }
}

impl ::std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Clone)]
pub struct IPFSHash(String);

impl From<String> for IPFSHash {
    fn from(s: String) -> Self {
        IPFSHash(s)
    }
}

impl<'a> From<&'a str> for IPFSHash {
    fn from(s: &str) -> Self {
        String::from(s).into()
    }
}

impl Deref for IPFSHash {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::std::fmt::Display for IPFSHash {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Clone)]
pub struct IPNSHash(String);

impl From<String> for IPNSHash {
    fn from(s: String) -> Self {
        IPNSHash(s)
    }
}

impl<'a> From<&'a str> for IPNSHash {
    fn from(s: &str) -> Self {
        String::from(s).into()
    }
}

impl Deref for IPNSHash {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::std::fmt::Display for IPNSHash {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        self.0.fmt(f)
    }
}

/// TODO: A String as mimetype... we can do this better!
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MimeType(Mime);

impl From<Mime> for MimeType {
    fn from(m: Mime) -> Self {
        MimeType(m)
    }
}

impl Deref for MimeType {
    type Target = Mime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::std::fmt::Display for MimeType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        self.0.fmt(f)
    }
}


//
// TODO: Remove code below as soon as "mime" has serde support
//

use std::fmt;
use std::str::FromStr;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

impl Serialize for MimeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for MimeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = MimeType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid MIME type")
            }

            fn visit_str<E>(self, value: &str) -> Result<MimeType, E>
                where E: de::Error,
            {
                Mime::from_str(value)
                    .map(MimeType)
                    .map_err(E::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

