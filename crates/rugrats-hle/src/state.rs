//! Saving and restoring a machine: a small little-endian codec and the
//! machine's own parts (CPU, GTE, memory, heap, interrupt timing). Devices
//! save themselves with the same codec.

/// Appends values.
#[derive(Default)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn u16(&mut self, v: u16) {
        self.0.extend(v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.0.extend(v.to_le_bytes());
    }
    pub fn bool(&mut self, v: bool) {
        self.u8(v as u8);
    }
    /// A length-prefixed byte string.
    pub fn bytes(&mut self, v: &[u8]) {
        self.u32(v.len() as u32);
        self.0.extend(v);
    }
    pub fn u16s(&mut self, v: &[u16]) {
        self.u32(v.len() as u32);
        for &x in v {
            self.u16(x);
        }
    }
    pub fn u32s(&mut self, v: &[u32]) {
        self.u32(v.len() as u32);
        for &x in v {
            self.u32(x);
        }
    }
}

/// Reads values back; any shortfall is an error.
pub struct Reader<'a>(pub &'a [u8]);

#[derive(Debug)]
pub struct StateError(pub String);

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "state: {}", self.0)
    }
}

impl std::error::Error for StateError {}

pub type Result<T> = std::result::Result<T, StateError>;

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.0.len() < n {
            return Err(StateError(format!("wanted {n} bytes, {} left", self.0.len())));
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn bool(&mut self) -> Result<bool> {
        Ok(self.u8()? != 0)
    }
    pub fn bytes(&mut self) -> Result<Vec<u8>> {
        let n = self.u32()? as usize;
        Ok(self.take(n)?.to_vec())
    }
    pub fn u16s(&mut self) -> Result<Vec<u16>> {
        let n = self.u32()? as usize;
        (0..n).map(|_| self.u16()).collect()
    }
    pub fn u32s(&mut self) -> Result<Vec<u32>> {
        let n = self.u32()? as usize;
        (0..n).map(|_| self.u32()).collect()
    }
    /// A tag written by `Writer::bytes(tag)`, checked.
    pub fn expect(&mut self, tag: &[u8]) -> Result<()> {
        let got = self.bytes()?;
        if got != tag {
            return Err(StateError(format!(
                "expected section {:?}, found {:?}",
                String::from_utf8_lossy(tag),
                String::from_utf8_lossy(&got)
            )));
        }
        Ok(())
    }
}
