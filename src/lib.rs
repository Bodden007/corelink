mod core;

use core::config::parser::parse_from_pointer;

#[unsafe(no_mangle)]
pub extern "C" fn corelink_configure(config_ptr: *const u8) -> i32 {
    match parse_from_pointer(config_ptr) {
        Ok(config) => {
            // FIXME: Временный диагностический вывод.
            // Удалить после подтверждения, что CoreLinkConfig
            // корректно создаётся из FFI-буфера.
            println!(
                "IP: {}.{}.{}.{}",
                config.ip[0], config.ip[1], config.ip[2], config.ip[3]
            );

            println!("Port: {}", config.port);
            println!("StartAddress: {}", config.start_address);
            println!("PollIntervalMs: {}", config.poll_interval_ms);
            println!("TypeMapLength: {}", config.type_map.len());
            println!("TypeMap: {:?}", config.type_map);

            0
        }

        Err(result) => result,
    }
}
