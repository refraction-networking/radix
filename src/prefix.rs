
use std::net::IpAddr;
use std::str::FromStr;
use std::fmt;

use failure::Error;

#[derive(Debug,PartialEq)]
pub struct CidrPrefix {
    pub net:    IpAddr,  // v4 or v6
    pub prefix: u8,
}

// Parse from e.g. "10.0.0.0/8" or "::1/64" into a CidrPrefix
impl FromStr for CidrPrefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error>
    {
        let parts: Vec<&str> = s.splitn(2, "/").collect();

        if parts.len() != 2 {
            return Err(format_err!("Does not contain a /"));
        }

        let net: IpAddr = parts[0].parse()?;
        let prefix_len = parts[1].parse::<u8>()?;

        // Check if prefix_len makes sense
        let max_plen = match net {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };

        if prefix_len > max_plen {
            return Err(format_err!("Prefix length too large"));
        }

        Ok(CidrPrefix { net:    net,
                        prefix: prefix_len })
    }
}

// Print as "10.0.0.0/8"
impl fmt::Display for CidrPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.net, self.prefix)
    }
}

#[cfg(test)]
mod tests {
    use prefix::CidrPrefix;
    #[test]
    fn test_v4() {
        let cidr: CidrPrefix = "127.0.0.0/8".parse().unwrap();

        let expected = CidrPrefix { net: "127.0.0.0".parse().unwrap(),
                                    prefix: 8 };

        assert_eq!(cidr, expected);
    }

    #[test]
    fn test_v6() {
        let cidr = "::1/128".parse::<CidrPrefix>().unwrap();

        let expected = CidrPrefix { net: "::1".parse().unwrap(),
                                    prefix: 128 };
        assert_eq!(cidr, expected);
    }

    #[test]
    fn test_invalid() {
        assert!("10.1.2.256/32".parse::<CidrPrefix>().is_err());
        assert!("10.1.0.0/33".parse::<CidrPrefix>().is_err());
        assert!("10.1.2.3/".parse::<CidrPrefix>().is_err());
        assert!("10.1.2.3".parse::<CidrPrefix>().is_err());
        assert!("/8".parse::<CidrPrefix>().is_err());

        // Invalid v6
        assert!("1:2:3:4::5:6:7:8/8".parse::<CidrPrefix>().is_err());
        assert!("1::3/129".parse::<CidrPrefix>().is_err());
        assert!("/100".parse::<CidrPrefix>().is_err());
    }
}
