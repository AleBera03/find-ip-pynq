use std::fmt::Display;
use anyhow::{Result, anyhow};

/// Ip struct which implements a simple ipv4 tool with following functionalities
#[derive(PartialEq, Eq)]
pub struct Ip{
    address: u32,
    bit_netmask: u8
}
pub struct Subnet{
    pub network_addr: Ip,
    pub min_host: Ip,
    pub max_host: Ip,
    pub broadcast_host: Ip
}


impl Ip {
    pub fn new(o1: u8, o2: u8, o3: u8, o4:u8, bit_nm: u8) -> Result<Self>{
        let new = Ip {
            address: ((o1 as u32) << 24) | ((o2 as u32) << 16) | ((o3 as u32) << 8) | (o4 as u32),
            bit_netmask: bit_nm
        };
        if !new.check_host() {
            return Err(anyhow!("host address is wrong"));
        };
        Ok(new)
    }
    fn serialize(addr: u32, bit_nm: u8) -> Result<Self>{
        let new = Ip { address: addr, bit_netmask: bit_nm };
        if !new.check_host(){
            return Err(anyhow!("host address is wrong"));
        };
        Ok(new)
    }
    /// from octale 0 to octale 3
    /// 
    /// 192.168.137.1
    /// 
    /// 1   2   3   4
    pub fn set_oct(&mut self, pos: usize, val: u8) -> Result<()>{
        if pos > 3 {
            return Err(anyhow!("octal index must be from 0 to 3"));
        };
        let shift = (3 - pos) * 8;
        let clean_mask = !(0xFF << shift);
        self.address = (self.address & clean_mask) | ((val as u32) << shift);
        Ok(())
    }
    pub fn get_oct(&self, pos: usize) -> Result<u8>{
        if pos > 3 {
            return Err(anyhow!("octal index must be from 0 to 3"));
        };
        let shift = (3 - pos) * 8;
        let mask = 0xFF << shift;
        let res = (self.address & mask) >> shift;
        Ok(res as u8)
    }
    /// return true if address is valid
    fn check_host(&self) -> bool{
        // check correctness of netmask
        if self.bit_netmask >= 31{ // point to point connession are not included for now
            return false;
        }
        let netmask: u32 = 0xFFFFFFFF << (32 - self.bit_netmask);
        if netmask == 0 {
            return false;
        }
        if netmask & (!netmask >> 1) == 1 {
            return false;
        }
        // check if address is a host
        let subnet: Subnet;
        let res_subnet = self.compute_subnet();
        match res_subnet {
            Ok(s) => {
                if *self == s.network_addr || *self == s.broadcast_host{
                    return false;
                }
                subnet = s;
            },
            Err(_) => {
                return false;
            }
        }
        // check if associated network address is correct
        let pos = self.bit_netmask / 8 + 1;
        let rem = self.bit_netmask % 8;
        let exp = 8 - rem;
        let mul = (2 as u32).pow(exp as u32);
        let res = subnet.network_addr.get_oct((pos-1) as usize);
        match res {
            Ok(o) => {
                if (o as u32 % mul) != 0 {
                    return false;
                };
            },
            Err(_) => {return false;}
        };
        true
    }
    pub fn to_string(&self) -> String{
        let mut shift: usize;
        let mut mask: u32;
        let mut res = String::new();
        for pos in 0..=3 {
            shift = (3 - pos)*8;
            mask = 0xFF << shift;
            res = concat_string(res, ((self.address & mask) >> shift).to_string());
            res = concat_string(res, String::from("."));
        };
        match res.pop() {
            Some(_) => res,
            None => "NaN".to_string()
        }
    }
    /// compute min, max, broadcast ipv4 addresses of given **network address**
    /// 
    /// which is &self
    pub fn compute_subnet(&self) -> Result<Subnet>{
        let netmask: u32 = 0xFFFFFFFF << (32 -self.bit_netmask);
        let na = self.address & netmask;
        let bh = na | !(netmask);
        let minh = na + 1;
        let maxh = bh - 1;
        Ok (
            Subnet {
                network_addr: Ip { address: na, bit_netmask: self.bit_netmask },
                min_host: Ip { address: minh, bit_netmask: self.bit_netmask },
                max_host: Ip { address: maxh, bit_netmask: self.bit_netmask },
                broadcast_host: Ip { address: bh, bit_netmask: self.bit_netmask }
            }
        )
    }
}

impl Iterator for Ip {
    type Item = Ip;
    fn next(&mut self) -> Option<Self::Item> {
        let subnet = self.compute_subnet().ok()?;
        if self.address >= subnet.broadcast_host.address {
            return None;
        }
        self.address += 1;
        
        Some(Ip {
            address: self.address,
            bit_netmask: self.bit_netmask,
        })
    }
}

impl Display for Ip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Display for Subnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.network_addr)?;
        writeln!(f, "{}", self.min_host)?;
        writeln!(f, "{}", self.max_host)?;
        writeln!(f, "{}", self.broadcast_host)
    }
}

fn concat_string(a: String, b: String) -> String {
    a + &b
}