use std::{
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NoHash<T>(u64, PhantomData<T>);

pub type BuildNoHashDefault<T> = BuildHasherDefault<NoHash<T>>;

impl<T> Hasher for NoHash<T> {
    fn write(&mut self, _: &[u8]) {
        unimplemented!("Invalid use of NoHashHasher")
    }

    fn write_u8(&mut self, n: u8) {
        self.0 = u64::from(n)
    }
    fn write_u16(&mut self, n: u16) {
        self.0 = u64::from(n)
    }
    fn write_u32(&mut self, n: u32) {
        self.0 = u64::from(n)
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n
    }
    fn write_usize(&mut self, _: usize) {
        unimplemented!("Invalid use of NoHashHasher")
    }

    fn write_i8(&mut self, n: i8) {
        self.0 = n as u64
    }
    fn write_i16(&mut self, n: i16) {
        self.0 = n as u64
    }
    fn write_i32(&mut self, n: i32) {
        self.0 = n as u64
    }
    fn write_i64(&mut self, n: i64) {
        self.0 = n as u64
    }
    fn write_isize(&mut self, _: isize) {
        unimplemented!("Invalid use of NoHashHasher")
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let mut h1 = NoHash::<u8>::default();
        h1.write_u8(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_u16() {
        let mut h1 = NoHash::<u16>::default();
        h1.write_u16(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_u32() {
        let mut h1 = NoHash::<u32>::default();
        h1.write_u32(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_u64() {
        let mut h1 = NoHash::<u32>::default();
        h1.write_u64(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_i8() {
        let mut h1 = NoHash::<i8>::default();
        h1.write_i8(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_i16() {
        let mut h1 = NoHash::<i16>::default();
        h1.write_i16(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_i32() {
        let mut h1 = NoHash::<i32>::default();
        h1.write_i32(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    fn test_i64() {
        let mut h1 = NoHash::<i32>::default();
        h1.write_i64(42);
        assert_eq!(42, h1.finish());
    }

    #[test]
    #[should_panic]
    fn test_usize() {
        let mut h1 = NoHash::<i32>::default();
        h1.write_usize(42);
    }

    #[test]
    #[should_panic]
    fn test_isize() {
        let mut h1 = NoHash::<i32>::default();
        h1.write_isize(42);
    }

    #[test]
    #[should_panic]
    fn test_bytes() {
        let mut h1 = NoHash::<[u8; 4]>::default();
        h1.write(&[0, 0, 0, 0]);
    }
}
