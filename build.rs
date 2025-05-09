#[cfg(windows)]
fn main() {
    
    println!("cargo:rustc-link-search=native=C:\\Program Files\\Npcap\\WpdPack\\Lib\\x64");
     // Link the Packet.lib static library
     println!("cargo:rustc-link-lib=static=Packet");

     // Also link any other required libraries
     println!("cargo:rustc-link-lib=ws2_32");
     println!("cargo:rustc-link-lib=iphlpapi");
}
#[cfg(unix)]
fn main() {}