/// Конфигурация CoreLink, принадлежащая Rust.
///
/// Все данные копируются из FFI-буфера.
/// После завершения Configure структура не зависит от памяти C#.
pub struct CoreLinkConfig {
    pub ip: [u8; 4],
    pub port: u16,
    pub start_address: u16,
    pub poll_interval_ms: i32,
    pub type_map: Vec<u8>,
}
