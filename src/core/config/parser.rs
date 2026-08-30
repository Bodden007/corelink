use super::model::CoreLinkConfig;

const HEADER_SIZE: usize = 20;
const MAX_CONFIG_SIZE: usize = 65_536;

pub fn parse_from_pointer(config_ptr: *const u8) -> Result<CoreLinkConfig, i32> {
    if config_ptr.is_null() {
        return Err(4);
    }

    let total_length = read_total_length(config_ptr)?;

    if total_length < HEADER_SIZE || total_length > MAX_CONFIG_SIZE {
        return Err(4);
    }

    // SAFETY:
    // C# удерживает массив pinned на всё время FFI-вызова.
    // Rust использует этот slice только для чтения и копирования.
    let data = unsafe { std::slice::from_raw_parts(config_ptr, total_length) };

    parse_from_slice(data)
}

fn parse_from_slice(data: &[u8]) -> Result<CoreLinkConfig, i32> {
    let ip_bytes = get_range(data, 4, 4)?;

    let ip = [ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]];

    let port = read_u16(data, 8)?;
    let start_address = read_u16(data, 10)?;
    let poll_interval_ms = read_i32(data, 12)?;
    let type_map_length = read_i32(data, 16)?;

    if poll_interval_ms <= 0 || type_map_length <= 0 {
        return Err(4);
    }

    let type_map_length = usize::try_from(type_map_length).map_err(|_| 4)?;

    let expected_length = HEADER_SIZE.checked_add(type_map_length).ok_or(4)?;

    if expected_length != data.len() {
        return Err(4);
    }

    let type_map = get_range(data, HEADER_SIZE, type_map_length)?.to_vec();

    Ok(CoreLinkConfig {
        ip,
        port,
        start_address,
        poll_interval_ms,
        type_map,
    })
}

fn read_total_length(config_ptr: *const u8) -> Result<usize, i32> {
    if config_ptr.is_null() {
        return Err(4);
    }

    // SAFETY:
    // FFI-контракт гарантирует минимум 4 байта заголовка.
    let raw = unsafe { std::ptr::read_unaligned(config_ptr.cast::<u32>()) };

    Ok(u32::from_le(raw) as usize)
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, i32> {
    let bytes = get_range(data, offset, 2)?;

    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_i32(data: &[u8], offset: usize) -> Result<i32, i32> {
    let bytes = get_range(data, offset, 4)?;

    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn get_range(data: &[u8], offset: usize, length: usize) -> Result<&[u8], i32> {
    let end = offset.checked_add(length).ok_or(4)?;

    data.get(offset..end).ok_or(4)
}
