const DTLS_CT_CHANGE_CIPHER_SPEC: u8 = 20;
const DTLS_CT_ALERT: u8 = 21;
const DTLS_CT_HANDSHAKE: u8 = 22;
const DTLS_CT_APPLICATION_DATA: u8 = 23;

const DTLS_VERSION_1_2: [u8; 2] = [0xFE, 0xFD];
const DTLS_VERSION_1_0: [u8; 2] = [0xFE, 0xFF];

const DTLS_CONFIRM_THRESHOLD: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtlsTag {
    // Number of DTLS application data records observed (the data channel in flight)
    pub app_data_records: usize,
    // Whether a DTLS handshake was also seen (not required to be tagged)
    pub saw_handshake: bool,
}

fn dtls_content_type(buf: &[u8]) -> Option<u8> {
    if buf.len() < 13 {
        return None;
    }

    let content_type = buf[0];
    if !matches!(content_type,
        DTLS_CT_CHANGE_CIPHER_SPEC | DTLS_CT_ALERT | DTLS_CT_HANDSHAKE | DTLS_CT_APPLICATION_DATA) {
        return None;
    }

    let version = &buf[1..3];
    if version != DTLS_VERSION_1_2 && version != DTLS_VERSION_1_0 {
        return None;
    }

    Some(content_type)
}

pub fn tag(c2s: &[Vec<u8>], s2c: &[Vec<u8>]) -> Option<DtlsTag> {
    let mut app_data_records = 0usize;
    let mut saw_handshake = false;

    for datagram in c2s.iter().chain(s2c.iter()) {
        if let Some(ct) = dtls_content_type(datagram) {
            match ct {
                DTLS_CT_APPLICATION_DATA => app_data_records += 1,
                DTLS_CT_HANDSHAKE => saw_handshake = true,
                _ => {}
            }
        }
    }
    
    if app_data_records >= DTLS_CONFIRM_THRESHOLD {
        Some(DtlsTag { app_data_records, saw_handshake })
    } else {
        None
    }
}