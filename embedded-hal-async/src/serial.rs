//! Serial interface

use core::future::Future;
pub use embedded_hal::serial::{Error, ErrorKind, ErrorType};

/// Read half of a serial interface
///
/// Some serial interfaces support different data sizes (8 bits, 9 bits, etc.);
/// This can be encoded in this trait via the `Word` type parameter.
pub trait Read<Word: 'static + Copy = u8>: ErrorType {
    /// Future returned by the `read` method.
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Reads words from the serial interface into the supplied slice.
    fn read<'a>(&'a mut self, read: &'a mut [Word]) -> Self::ReadFuture<'a>;
}

impl<T: Read<Word>, Word: 'static + Copy> Read<Word> for &mut T {
    type ReadFuture<'a> = T::ReadFuture<'a> where Self: 'a;

    fn read<'a>(&'a mut self, read: &'a mut [Word]) -> Self::ReadFuture<'a> {
        T::read(self, read)
    }
}

/// Write half of a serial interface
pub trait Write<Word: 'static + Copy = u8>: ErrorType {
    /// Future returned by the `write` method.
    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Writes a single word to the serial interface
    fn write<'a>(&'a mut self, words: &'a [Word]) -> Self::WriteFuture<'a>;

    /// Future returned by the `flush` method.
    type FlushFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Ensures that none of the previously written words are still buffered
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>;
}

impl<T: Write<Word>, Word: 'static + Copy> Write<Word> for &mut T {
    type WriteFuture<'a> = T::WriteFuture<'a> where Self: 'a;

    fn write<'a>(&'a mut self, words: &'a [Word]) -> Self::WriteFuture<'a> {
        T::write(self, words)
    }

    type FlushFuture<'a> = T::FlushFuture<'a> where Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        T::flush(self)
    }
}
