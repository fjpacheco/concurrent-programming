pub const ID_CORDINADOR_INICIAL: u8 = 1;
pub const SALDO_INICIAL: u32 = 10000;
pub const MAX_UDP_SIZE: usize = 14;
pub const CANT_MAX_NODOS: u8 = 3;
pub const TIMEOUT_OK_BULLY_MILLIS: u64 = 10000;

pub fn id_to_addr_read_data(id: u8) -> String {
    "127.0.0.1:1235".to_owned() + &id.to_string()
}

pub fn id_to_addr_write_data(id: u8) -> String {
    "127.0.0.1:1236".to_owned() + &id.to_string()
}

pub fn id_to_ctrladdr(id: u8) -> String {
    "127.0.0.1:1234".to_owned() + &id.to_string()
}

pub fn id_to_addr_write_bully(id: u8) -> String {
    "127.0.0.1:1243".to_owned() + &id.to_string()
}

pub fn id_to_addr_read_bully(id: u8) -> String {
    "127.0.0.1:1242".to_owned() + &id.to_string()
}

// echo -e '\x00''\x10' | nc -u 127.0.0.1 12341
