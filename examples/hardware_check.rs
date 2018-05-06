//! This example performs a test using real hardware ports.
//!
//! This tool serves as a debugging aid when encountering errors using `serialport-rs`. It should
//! expose any kernel or driver bugs that your system may have by running physical ports through
//! many configurations.
//!
//! To run this you will need two ports on your system that have been connected together such that
//! transmissions on one are received by the other and vice-versa. This program will then test
//! configuring all possible settings per-port before moving on to making sure these have real-world
//! implications by writing data on one port and reading it on another.

extern crate argparse;
extern crate serialport;

use argparse::{ArgumentParser, Store};

use std::time::Duration;
use std::str;

use serialport::prelude::*;

fn main() {
    let mut port1_name = "".to_string();
    let mut port2_name = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Test serial ports");
        ap.refer(&mut port1_name)
            .add_argument("port1", Store, "Port 1 name")
            .required();
        ap.refer(&mut port2_name)
            .add_argument("port2", Store, "Port 2 name");
        ap.parse_args_or_exit();
    }

    // Run single-port tests on port1
    let mut port1 = serialport::open(&port1_name).unwrap();
    test_single_port(&mut *port1);

    if port2_name != "" {
        // Run single-port tests on port2
        let mut port2 = serialport::open(&port2_name).unwrap();
        test_single_port(&mut *port2);

        // Test loopback pair
        test_dual_ports(&mut *port1, &mut *port2);
    }
}

macro_rules! baud_rate_check {
    ( $port:ident, $baud:expr ) => {
        let baud_rate = $baud;
        if let Err(e) = $port.set_baud_rate(baud_rate) {
            println!("  {:?}: FAILED ({})", baud_rate, e);
        }
        match $port.baud_rate() {
            Err(_) => println!("  {:?}: FAILED (error retrieving baud rate)", baud_rate),
            Ok(r) if r != baud_rate => println!("  {:?}: FAILED (baud rate {:?} does not match set baud rate {:?})",
                baud_rate,
                r,
                baud_rate),
            Ok(_) => println!("  {:?}: success", baud_rate),
        }
    };
}

macro_rules! data_bits_check {
    ( $port:ident, $data_bits:path ) => {
        let data_bits = $data_bits;
        if let Err(e) = $port.set_data_bits(data_bits) {
            println!("  {:?}: FAILED ({})", data_bits, e);
        } else {
            match $port.data_bits() {
                Err(_) => println!("FAILED to retrieve data bits"),
                Ok(r) if r != data_bits => println!("  {:?}: FAILED (data bits {:?} does not match set data bits {:?})",
                    data_bits,
                    r,
                    data_bits),
                Ok(_) => println!("  {:?}: success", data_bits),
            }
        }
    };
}

macro_rules! flow_control_check {
    ( $port:ident, $flow_control:path ) => {
        let flow_control = $flow_control;
        if let Err(e) = $port.set_flow_control(flow_control) {
            println!("  {:?}: FAILED ({})", flow_control, e);
        } else {
            match $port.flow_control() {
                Err(_) => println!("FAILED to retrieve flow control"),
                Ok(r) if r != flow_control => println!("  {:?}: FAILED (flow control {:?} does not match set flow control {:?})",
                    flow_control,
                    r,
                    flow_control),
                Ok(_) => println!("  {:?}: success", flow_control),
            }
        }
    };
}

macro_rules! parity_check {
    ( $port:ident, $parity:path ) => {
        let parity = $parity;
        if let Err(e) = $port.set_parity(parity) {
            println!("  {:?}: FAILED ({})", parity, e);
        } else {
            match $port.parity() {
                Err(_) => println!("FAILED to retrieve parity"),
                Ok(r) if r != parity => println!("  {:?}: FAILED (parity {:?} does not match set parity {:?})",
                    parity,
                    r,
                    parity),
                Ok(_) => println!("  {:?}: success", parity),
            }
        }
    };
}

macro_rules! stop_bits_check {
    ( $port:ident, $stop_bits:path ) => {
        let stop_bits = $stop_bits;
        if let Err(e) = $port.set_stop_bits(stop_bits) {
            println!("  {:?}: FAILED ({})", stop_bits, e);
        } else {
            match $port.stop_bits() {
                Err(_) => println!("FAILED to retrieve stop bits"),
                Ok(r) if r != stop_bits => println!("FAILED, stop bits {:?} does not match set stop bits {:?}",
                    r,
                    stop_bits),
                Ok(_) => println!("  {:?}: success", stop_bits),
            }
        }
    };
}

fn test_single_port(port: &mut serialport::SerialPort) {
    println!("Testing '{}':", port.name().unwrap());

    // Test setting standard baud rates
    println!("Testing baud rates...");
    baud_rate_check!(port, 9600);
    baud_rate_check!(port, 38400);
    baud_rate_check!(port, 115200);

    // Test setting non-standard baud rates
    println!("Testing non-standard baud rates...");
    baud_rate_check!(port, 10000);
    baud_rate_check!(port, 600000);
    baud_rate_check!(port, 1800000);

    // Test setting the data bits
    println!("Testing data bits...");
    data_bits_check!(port, DataBits::Five);
    data_bits_check!(port, DataBits::Six);
    data_bits_check!(port, DataBits::Seven);
    data_bits_check!(port, DataBits::Eight);

    // Test setting flow control
    println!("Testing flow control...");
    flow_control_check!(port, FlowControl::Software);
    flow_control_check!(port, FlowControl::Hardware);
    flow_control_check!(port, FlowControl::None);

    // Test setting parity
    println!("Testing parity...");
    parity_check!(port, Parity::Odd);
    parity_check!(port, Parity::Even);
    parity_check!(port, Parity::None);

    // Test setting stop bits
    println!("Testing stop bits...");
    stop_bits_check!(port, StopBits::Two);
    stop_bits_check!(port, StopBits::One);

}

fn test_dual_ports(port1: &mut serialport::SerialPort, port2: &mut serialport::SerialPort) {
    println!("Testing paired ports '{}' and '{}':", port1.name().unwrap(), port2.name().unwrap());

    // Make sure both ports are set to sane defaults
    let mut port_settings: SerialPortSettings = Default::default();
    port_settings.timeout = Duration::from_millis(100);
    port_settings.baud_rate = 115200;
    port1.set_all(&port_settings).expect("Resetting port1 to sane defaults failed");
    port2.set_all(&port_settings).expect("Resetting port2 to sane defaults failed");

    let msg = "Test Message";
    let mut buf = [0u8; 12];

    // Test sending strings from port1 to port2
    println!("  Transmitting from {} to {}...", port1.port_name().unwrap(), port2.port_name().unwrap());
    port1.set_baud_rate(2000000).expect("Setting port1's baud rate to 2000000 failed");
    port2.set_baud_rate(2000000).expect("Setting port2's baud rate to 2000000 failed");
    print!("     At 2000000,8,n,1,noflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(115200).expect("Setting port1's baud rate to 115200 failed");
    port2.set_baud_rate(115200).expect("Setting port2's baud rate to 115200 failed");
    print!("     At 115200,8,n,1,noflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(57600).expect("Setting port1's baud rate to 57600 failed");
    port2.set_baud_rate(57600).expect("Setting port2's baud rate to 57600 failed");
    print!("     At 57600,8,n,1,noflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(10000).expect("Setting port1's baud rate to 9600 failed");
    port2.set_baud_rate(10000).expect("Setting port2's baud rate to 9600 failed");
    print!("     At 10000,8,n,1,noflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(9600).expect("Setting port1's baud rate to 9600 failed");
    port2.set_baud_rate(9600).expect("Setting port2's baud rate to 9600 failed");
    print!("     At 9600,8,n,1,noflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }

    // Test flow control
    port1.set_flow_control(FlowControl::Software).unwrap();
    port2.set_flow_control(FlowControl::Software).unwrap();
    print!("     At 9600,8,n,1,softflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_flow_control(FlowControl::Hardware).unwrap();
    port2.set_flow_control(FlowControl::Hardware).unwrap();
    print!("     At 9600,8,n,1,hardflow...");
    let nbytes = port1.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port2.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }

    // Test sending strings from port2 to port1
    port1.set_flow_control(FlowControl::None).unwrap();
    port2.set_flow_control(FlowControl::None).unwrap();
    port1.set_baud_rate(2000000).expect("Setting port1's baud rate to 2000000 failed");
    port2.set_baud_rate(2000000).expect("Setting port2's baud rate to 2000000 failed");
    println!("  Transmitting from {} to {}...", port2.port_name().unwrap(), port1.port_name().unwrap());
    print!("     At 2000000,8,n,1,noflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(115200).expect("Setting port1's baud rate to 115200 failed");
    port2.set_baud_rate(115200).expect("Setting port2's baud rate to 115200 failed");
    print!("     At 115200,8,n,1,noflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(57600).expect("Setting port1's baud rate to 57600 failed");
    port2.set_baud_rate(57600).expect("Setting port2's baud rate to 57600 failed");
    print!("     At 57600,8,n,1,noflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(10000).expect("Setting port1's baud rate to 10000 failed");
    port2.set_baud_rate(10000).expect("Setting port2's baud rate to 10000 failed");
    print!("     At 10000,8,n,1,noflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_baud_rate(9600).expect("Setting port1's baud rate to 9600 failed");
    port2.set_baud_rate(9600).expect("Setting port2's baud rate to 9600 failed");
    print!("     At 9600,8,n,1,noflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }

    // Test flow control
    port1.set_flow_control(FlowControl::Software).unwrap();
    port2.set_flow_control(FlowControl::Software).unwrap();
    print!("     At 9600,8,n,1,softflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
    port1.set_flow_control(FlowControl::Hardware).unwrap();
    port2.set_flow_control(FlowControl::Hardware).unwrap();
    print!("     At 9600,8,n,1,hardflow...");
    let nbytes = port2.write(msg.as_bytes()).expect("Unable to write bytes.");
    assert_eq!(nbytes,
               msg.len(),
               "Write message length differs from sent message.");
    if port1.read_exact(&mut buf).is_err() {
        println!("FAILED");
    } else {
        assert_eq!(str::from_utf8(&buf).unwrap(),
                   msg,
                   "Received message does not match sent");
        println!("success");
    }
}
