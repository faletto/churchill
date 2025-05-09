use clap::Parser;
use ctrlc;
use dhcp4r::packet::Packet;
use dhcp4r::options::{DhcpOption, MessageType as DhcpMessageType};
use getifaddrs::getifaddrs;
use rand::Rng;
use std::io::{self, Write};
use std::net::UdpSocket;
use std::time::Duration;



// Information displayed in the CLI help menu
#[derive(Parser, Debug)]
#[command(
    name = "churchill",
    version = "1.0.0",
    about = "DHCP Discover"
)]
struct Cli {
    /// Number of packets to send (0 = unlimited)
    #[arg(short, long, default_value_t = 0)]
    number: i32,
    /// Delay between packets in milliseconds (defaults to instantaneous)
    #[arg(short, long, default_value_t = 0)]
    delay: i32,
    /// Set custom address to bind address to
    #[arg(short,long,default_value_t = String::from("0.0.0.0"))]
    address: String

}

fn main() {
    // Controls behavior when the user sends an interrupt command (CTRL + C)
    ctrlc::set_handler(move || {
        print!("\n[!] Stopped by user.");
        std::process::exit(0);
    })
    .expect("[!] Failure setting CTRL+C handler.");

    // Checks if user has elevated privileges (needed to open a socket)
    if !is_elevated() {
        println!("[!] Not elevated. Try running 'sudo churchill' if on MacOS/Linux, or running as administrator if on Windows.");
        std::process::exit(0);
    }

    // Parses command-line arguments
    let cli = Cli::parse();

    let mut addr = cli.address;
    if addr == "0.0.0.0" {
        
    }
    
    let mut packet = Packet {
        broadcast : true,
        reply : false,
        hops: 0,
        xid: 0x12345678,
        secs: 0,
        ciaddr: std::net::Ipv4Addr::UNSPECIFIED,
        yiaddr: std::net::Ipv4Addr::UNSPECIFIED,
        siaddr: std::net::Ipv4Addr::UNSPECIFIED,
        giaddr: std::net::Ipv4Addr::UNSPECIFIED,
        chaddr: rand_mac(),
        options: vec![
            DhcpOption::DhcpMessageType(DhcpMessageType::Discover)
        ]
    };

    let mut buf = [0u8;1500];

    let socket = UdpSocket::bind(format!("{}:68",addr)).unwrap();
    socket.set_broadcast(true).ok();


    let number;
    if cli.number == 0 {
        number = String::from("Infinite");
    } else {
        number = cli.number.to_string()
    }

    let s;
    if cli.number == 1 {
        s = ""
    } else {
        s = "s"
    }

    println!("Sending {} DHCP Discover Packet{} on address {}", number,s,cli.address);

   
    // Counter for keeping track of packets sent
    let mut count = 0;
    // The cli.number == 0 is for infinite packets
    while count < cli.number || cli.number == 0 {
        // Randomizes source MAC address
        packet.chaddr = rand_mac();

        packet.xid = rand_u32();
        let encoded = packet.encode(&mut buf);
        socket.send_to(encoded, "255.255.255.255:67").expect("[!] Error sending packet out of socket");
        count += 1;
        print!(
            "{} [{}/{}]\r",
            progress_bar((count, cli.number)),
            count,
            cli.number
        );
        // Ensures output is shown, because apparently print!() doesn't do that on its own??
        io::stdout().flush().expect("[!] Error flushing output");
        if cli.delay > 0 {
            std::thread::sleep(Duration::from_millis(cli.delay as u64));
        }
    }
    print!("\n");
}

fn rand_mac() -> [u8;6] {
    let mut rng = rand::rng();
    // Creates buffer for MAC address
    let mut mac = [0u8; 6];
    // Fills buffer with random values
    rng.fill(&mut mac);
    // Ensures that the MAC address is unicast
    mac[0] &= 0b11111110;
    // There's probably a better way to do this
    return mac;
}

fn rand_u32() -> u32{
    let mut rng = rand::rng();

    let u: u32 = rng.random();

    return u;
}


// Returns a progress bar like [###>.......]
fn progress_bar((count, total): (i32, i32)) -> String {
    if total == 0 {
        return "[..........]".to_string();
    }
    let filled = (count * 10 / total) as usize;
    let empty = 10 - filled;
    format!("[{}>{}]", "#".repeat(filled), ".".repeat(empty))
}


#[cfg(target_family = "unix")]
fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(target_family = "windows")]
fn is_elevated() -> bool {
    // If the user can open a "net session", they are probably elevated
    // Windows is weird
    use std::process::Command;
    Command::new("net")
        .arg("session")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

