use anyhow::{anyhow};
use std::fmt::format;
use std::ops::Add;
use std::{process::Command};

mod gui;
mod ip;
use crate::ip::Ip;
use crate::gui::MyEguiApp;

use std::net::IpAddr;
use std::time::Duration;
use futures::future::join_all;
use rand::random;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};
use tokio::time;
use tokio::runtime::Runtime;

use pyo3::prelude::*;


#[pyfunction]
fn find_ip() -> PyResult<String> {
    let rt = Runtime::new().map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    
    rt.block_on(async {
        let base_ip = Ip::new(192, 168, 137, 1, 24)?;
        let client_v4 = Client::new(&Config::default())?;
        let client_v6 = Client::new(&Config::builder().kind(ICMP::V6).build())?;
        let mut tasks = Vec::new();
        for ip in  base_ip.into_iter(){
            match ip.to_string().parse() {
                Ok(IpAddr::V4(addr)) => {
                    tasks.push(tokio::spawn(ping(client_v4.clone(), IpAddr::V4(addr))))
                }
                Ok(IpAddr::V6(addr)) => {
                    tasks.push(tokio::spawn(ping(client_v6.clone(), IpAddr::V6(addr))))
                }
                Err(e) => println!("{} parse to ipaddr error: {}", ip, e),
            }
        }

        let succeds = join_all(tasks).await;
        let success = &mut false;
        for s in succeds {
            match s {
                Ok(option) => {
                    match option {
                        Some(ip) => {
                            println!("ip correctly pinged: {}", ip);
                            
                            // run netsh command
                            Command::new("netsh").
                                args([  "interface",
                                        "portproxy",
                                        "add",
                                        "v4tov4",
                                        format!("listenaddress={}", Ip::new(192, 168, 1, 228, 24)?).as_str(),
                                        "listenport=8080",
                                        format!("connectaddress={}", ip).as_str(),
                                        "connectport=8080"])
                                .output()?.stdout;

                            // run success gui
                            *success = true;
                            let native_options = eframe::NativeOptions::default();
                            eframe::run_native("RESULT REMOTE IP", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, url(ip.to_string()))))))?;

                            return Ok(ip.to_string());
                        },
                        None => {}
                    }
                },
                Err(e) => {
                    return Err(anyhow!(e));
                }
            }
        }

        if *success == false {
            let native_options = eframe::NativeOptions::default();
            eframe::run_native("RESULT REMOTE IP", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, "Connection failed".to_string())))))?;
        }

        Ok(String::new())
    }).map_err(|e: anyhow::Error| PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))
}

// print direct url for streaming
fn url(ip_str: String) -> String {
    format!("http://{ip_str}:8080")
}

// Ping an address 2 times，and print output message（interval 1s)
async fn ping(client: Client, addr: IpAddr) -> Option<IpAddr>{
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_secs(1));

    /*
    for idx in 0..2 {
        interval.tick().await;
        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(packet), dur)) => {
                println!(
                    "No.{}: {} bytes from {}: icmp_seq={} ttl={:?} time={:0.2?}",
                    idx,
                    packet.get_size(),
                    packet.get_source(),
                    packet.get_sequence(),
                    packet.get_ttl(),
                    dur
                );
                return Some(addr);
            },
            Ok((IcmpPacket::V6(packet), dur)) => {
                println!(
                "No.{}: {} bytes from {}: icmp_seq={} hlim={} time={:0.2?}",
                    idx,
                    packet.get_size(),
                    packet.get_source(),
                    packet.get_sequence(),
                    packet.get_max_hop_limit(),
                    dur
                )
                return Some(addr);
            },
            Err(e) => {
                println!("No.{}: {} ping {}", idx, pinger.host, e);
                return None;
            }
        };
    }
    */

    let mut success = false;
    for idx in 0..2 {
        interval.tick().await;
        if pinger.ping(PingSequence(idx), &payload).await.is_ok() {
            success = true; // Almeno un ping è andato a buon fine
        }
    }

    if success {
        Some(addr)
    } else {
        None
    }

}


#[pymodule]
fn set_ip_pynq(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(find_ip, m)?)?;
    Ok(())
}