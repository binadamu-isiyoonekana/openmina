use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Host {
    /// A DNS domain name, as '.' dot-separated labels.
    /// Non-ASCII labels are encoded in punycode per IDNA if this is the host of
    /// a special URL, or percent encoded for non-special URLs. Hosts for
    /// non-special URLs are also called opaque hosts.
    Domain(String),

    /// An IPv4 address.
    Ipv4(Ipv4Addr),

    /// An IPv6 address.
    Ipv6(Ipv6Addr),
}

mod binprot_impl {
    use super::*;
    use binprot::{BinProtRead, BinProtWrite};
    use binprot_derive::{BinProtRead, BinProtWrite};

    #[derive(BinProtWrite, BinProtRead)]
    enum HostKind {
        Domain,
        Ipv4,
        Ipv6,
    }

    impl BinProtWrite for Host {
        fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
            Ok(match self {
                Self::Domain(v) => {
                    HostKind::Domain.binprot_write(w)?;
                    v.binprot_write(w)?
                }
                Self::Ipv4(v) => {
                    HostKind::Ipv4.binprot_write(w)?;
                    for b in v.octets() {
                        b.binprot_write(w)?;
                    }
                }
                Self::Ipv6(v) => {
                    HostKind::Ipv6.binprot_write(w)?;
                    for b in v.segments() {
                        b.binprot_write(w)?;
                    }
                }
            })
        }
    }

    impl BinProtRead for Host {
        fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
        where
            Self: Sized,
        {
            let kind = HostKind::binprot_read(r)?;

            Ok(match kind {
                HostKind::Domain => {
                    // TODO(binier): maybe limit length?
                    Host::from_str(&String::binprot_read(r)?)
                        .map_err(|err| binprot::Error::CustomError(err.into()))?
                }
                HostKind::Ipv4 => {
                    let mut octets = [0; 4];
                    for i in 0..octets.len() {
                        octets[i] = u8::binprot_read(r)?;
                    }

                    Host::Ipv4(octets.into())
                }
                HostKind::Ipv6 => {
                    let mut segments = [0; 8];
                    for i in 0..segments.len() {
                        segments[i] = u16::binprot_read(r)?;
                    }

                    Host::Ipv6(segments.into())
                }
            })
        }
    }
}

impl FromStr for Host {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(url::Host::parse(s)?.into())
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        url::Host::from(self).fmt(f)
    }
}

impl From<url::Host> for Host {
    fn from(value: url::Host) -> Self {
        match value {
            url::Host::Domain(v) => Host::Domain(v),
            url::Host::Ipv4(v) => Host::Ipv4(v),
            url::Host::Ipv6(v) => Host::Ipv6(v),
        }
    }
}

impl<'a> From<&'a Host> for url::Host<&'a str> {
    fn from(value: &'a Host) -> Self {
        match value {
            Host::Domain(v) => url::Host::Domain(v),
            Host::Ipv4(v) => url::Host::Ipv4(*v),
            Host::Ipv6(v) => url::Host::Ipv6(*v),
        }
    }
}
