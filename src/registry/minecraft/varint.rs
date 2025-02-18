#[derive(Debug)]
pub enum VarIntError {
    Incomplete,
    TooLarge,
}

pub fn read_varint(bytes: &[u8]) -> Result<(i32, usize), VarIntError> {
    let mut result = 0;
    let mut position = 0;
    let mut shift = 0;

    loop {
        if position >= bytes.len() {
            return Err(VarIntError::Incomplete);
        }

        let current = bytes[position];
        result |= ((current & 0b0111_1111) as i32) << shift;
        position += 1;

        if current & 0b1000_0000 == 0 {
            break;
        }

        shift += 7;
        if shift >= 32 {
            return Err(VarIntError::TooLarge);
        }
    }

    Ok((result, position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_byte() {
        assert_eq!(read_varint(&[0x00]).unwrap(), (0, 1));
        assert_eq!(read_varint(&[0x01]).unwrap(), (1, 1));
        assert_eq!(read_varint(&[0x7f]).unwrap(), (127, 1));
    }

    #[test]
    fn test_two_bytes() {
        assert_eq!(read_varint(&[0x80, 0x01]).unwrap(), (128, 2));
        assert_eq!(read_varint(&[0xff, 0x01]).unwrap(), (255, 2));
    }

    #[test]
    fn test_protocol_version_759() {
        // Minecraft 1.19 protocol version (759)
        assert_eq!(read_varint(&[0xf7, 0x05]).unwrap(), (759, 2));
    }

    #[test]
    fn test_incomplete() {
        assert!(matches!(read_varint(&[0x80]), Err(VarIntError::Incomplete)));
    }

    #[test]
    fn test_too_large() {
        assert!(matches!(
            read_varint(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]),
            Err(VarIntError::TooLarge)
        ));
    }

    #[test]
    fn test_max_value() {
        // Test maximum valid 32-bit value
        assert_eq!(
            read_varint(&[0xff, 0xff, 0xff, 0xff, 0x07]).unwrap(),
            (2147483647, 5)
        );
    }

    #[test]
    fn test_real_packet() {
        // Test actual Minecraft handshake packet bytes
        let packet = [0x10, 0x00, 0xf7, 0x05, 0x09];
        let (length, pos1) = read_varint(&packet).unwrap();
        let (packet_id, pos2) = read_varint(&packet[pos1..]).unwrap();
        let (protocol, pos3) = read_varint(&packet[pos1 + pos2..]).unwrap();
        assert_eq!(length, 16); // Packet length
        assert_eq!(packet_id, 0); // Handshake packet ID
        assert_eq!(protocol, 759); // Protocol version
        assert_eq!(packet[pos1 + pos2 + pos3], 0x09); // String length
    }
}
