use near_primitives::types::Gas;

pub fn pretty_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let exp = (bytes as f64).log(1024.0).floor() as u32;
    let pre = "KMGTPE".as_bytes()[(exp - 1) as usize] as char;
    format!("{:.2} {}B", bytes as f64 / 1024f64.powi(exp as i32), pre)
}

pub fn pretty_gas(gas: Gas) -> String {
    let tera_gas = 1_000_000_000_000.0;

    format!("{:.4} TGas", gas as f64 / tera_gas)
}
