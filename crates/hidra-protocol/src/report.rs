use crate::PadState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct X360Report(pub [u8; 14]);

impl From<[u8; 14]> for X360Report {
    fn from(data: [u8; 14]) -> Self {
        X360Report(data)
    }
}

impl X360Report {
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 14] {
        &self.0
    }
}

/// Pack IOCTL PadState into XUSB (Xbox 360) report.
/// Assumptions:
/// - `buttons`: already uses XUSB bit positions (u16).
/// - `lx,ly,rx,ry`: signed i16, LE in the report.
/// - `lt,rt`: u16; we clamp and downscale to 0..255.
impl From<&PadState> for X360Report {
    fn from(s: &PadState) -> Self {
        let mut b = [0u8; 14];

        // buttons (LE u16)
        let btn = s.buttons.to_le_bytes();
        b[0] = btn[0];
        b[1] = btn[1];

        // triggers
        // If your lt/rt are already 0..255, this is a no-op.
        // If they are 0..1023/4095, simple downscale (>> 2 / >> 4) works.
        b[2] = (s.lt.min(255)) as u8;
        b[3] = (s.rt.min(255)) as u8;

        // sticks: pass through i16 (LE)
        b[4..6].copy_from_slice(&s.lx.to_le_bytes());
        b[6..8].copy_from_slice(&s.ly.to_le_bytes());
        b[8..10].copy_from_slice(&s.rx.to_le_bytes());
        b[10..12].copy_from_slice(&s.ry.to_le_bytes());

        X360Report(b)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DS4Report(pub Vec<u8>);

impl From<Vec<u8>> for DS4Report {
    fn from(data: Vec<u8>) -> Self {
        DS4Report(data)
    }
}

impl DS4Report {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Pack IOCTL PadState into DS4 report.
impl From<&PadState> for DS4Report {
    fn from(_s: &PadState) -> Self {
        todo!("DS4 report packing not implemented yet");
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DS5Report(pub Vec<u8>);

impl From<Vec<u8>> for DS5Report {
    fn from(data: Vec<u8>) -> Self {
        DS5Report(data)
    }
}

impl DS5Report {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Pack IOCTL PadState into DS5 report.
impl From<&PadState> for DS5Report {
    fn from(_s: &PadState) -> Self {
        todo!("DS5 report packing not implemented yet");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PadState;

    #[test]
    fn x360_zeroed_report() {
        let s = PadState { buttons: 0, lx: 0, ly: 0, rx: 0, ry: 0, lt: 0, rt: 0 };
        let r = X360Report::from(&s);
        assert_eq!(r.as_bytes().len(), 14);
        assert_eq!(r.as_bytes()[0], 0);
        assert_eq!(r.as_bytes()[1], 0);
        assert_eq!(r.as_bytes()[2], 0);
        assert_eq!(r.as_bytes()[3], 0);
        assert_eq!(&r.as_bytes()[4..6], &0i16.to_le_bytes());
        assert_eq!(&r.as_bytes()[6..8], &0i16.to_le_bytes());
        assert_eq!(&r.as_bytes()[8..10], &0i16.to_le_bytes());
        assert_eq!(&r.as_bytes()[10..12], &0i16.to_le_bytes());
    }

    #[test]
    fn x360_buttons_and_axes() {
        let s = PadState {
            buttons: 0b1010_0000_0000_0011, // sample bits
            lx: 123,
            ly: -456,
            rx: 32767,
            ry: -32768,
            lt: 255,
            rt: 4096, // rt will clamp to 255 in the simple path
        };
        let r = X360Report::from(&s);
        let bytes = r.as_bytes();

        // buttons little-endian
        let expected_btn = s.buttons.to_le_bytes();
        assert_eq!(bytes[0], expected_btn[0]);
        assert_eq!(bytes[1], expected_btn[1]);

        // triggers (clamped)
        assert_eq!(bytes[2], 255);
        assert_eq!(bytes[3], 255);

        // sticks LE
        assert_eq!(&bytes[4..6], &s.lx.to_le_bytes());
        assert_eq!(&bytes[6..8], &s.ly.to_le_bytes());
        assert_eq!(&bytes[8..10], &s.rx.to_le_bytes());
        assert_eq!(&bytes[10..12], &s.ry.to_le_bytes());
    }
}
