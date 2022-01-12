//! Serial Peripheral Interface

use core::future::Future;

pub use embedded_hal::spi::blocking::Operation;
pub use embedded_hal::spi::{
    Error, ErrorKind, ErrorType, Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3,
};

/// Read-only SPI
pub trait Read<Word: 'static + Copy = u8>: ErrorType {
    /// Future returned by the `read` method.
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Read `words` from the slave.
    ///
    /// The word value sent on MOSI during reading is implementation-defined,
    /// typically `0x00`, `0xFF`, or configurable.
    fn read<'a>(&'a mut self, words: &'a mut [Word]) -> Self::ReadFuture<'a>;

    /// Future returned by the `read_transaction` method.
    type ReadTransactionFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Read all slices in `words` from the slave as part of a single SPI transaction.
    ///
    /// The word value sent on MOSI during reading is implementation-defined,
    /// typically `0x00`, `0xFF`, or configurable.
    fn read_transaction<'a>(
        &'a mut self,
        words: &'a mut [&'a mut [Word]],
    ) -> Self::ReadTransactionFuture<'a>;
}

impl<T: Read<Word>, Word: 'static + Copy> Read<Word> for &mut T {
    type ReadFuture<'a>
    where
        Self: 'a,
    = T::ReadFuture<'a>;

    fn read<'a>(&'a mut self, words: &'a mut [Word]) -> Self::ReadFuture<'a> {
        T::read(self, words)
    }

    type ReadTransactionFuture<'a>
    where
        Self: 'a,
    = T::ReadTransactionFuture<'a>;

    fn read_transaction<'a>(
        &'a mut self,
        words: &'a mut [&'a mut [Word]],
    ) -> Self::ReadTransactionFuture<'a> {
        T::read_transaction(self, words)
    }
}

/// Write-only SPI
pub trait Write<Word: 'static + Copy = u8>: ErrorType {
    /// Future returned by the `write` method.
    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Write `words` to the slave, ignoring all the incoming words
    fn write<'a>(&'a mut self, words: &'a [Word]) -> Self::WriteFuture<'a>;

    /// Future returned by the `write_transaction` method.
    type WriteTransactionFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Write all slices in `words` to the slave as part of a single SPI transaction, ignoring all the incoming words
    fn write_transaction<'a>(
        &'a mut self,
        words: &'a [&'a [Word]],
    ) -> Self::WriteTransactionFuture<'a>;
}

impl<T: Write<Word>, Word: 'static + Copy> Write<Word> for &mut T {
    type WriteFuture<'a>
    where
        Self: 'a,
    = T::WriteFuture<'a>;

    fn write<'a>(&'a mut self, words: &'a [Word]) -> Self::WriteFuture<'a> {
        T::write(self, words)
    }

    type WriteTransactionFuture<'a>
    where
        Self: 'a,
    = T::WriteTransactionFuture<'a>;

    fn write_transaction<'a>(
        &'a mut self,
        words: &'a [&'a [Word]],
    ) -> Self::WriteTransactionFuture<'a> {
        T::write_transaction(self, words)
    }
}

/// Read-write SPI
pub trait ReadWrite<Word: 'static + Copy = u8>: Read<Word> + Write<Word> {
    /// Future returned by the `transfer` method.
    type TransferFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Write and read simultaneously. `write` is written to the slave on MOSI and
    /// words received on MISO are stored in `read`.
    ///
    /// It is allowed for `read` and `write` to have different lengths, even zero length.
    /// The transfer runs for `max(read.len(), write.len())` words. If `read` is shorter,
    /// incoming words after `read` has been filled will be discarded. If `write` is shorter,
    /// the value of words sent in MOSI after all `write` has been sent is implementation-defined,
    /// typically `0x00`, `0xFF`, or configurable.
    fn transfer<'a>(
        &'a mut self,
        read: &'a mut [Word],
        write: &'a [Word],
    ) -> Self::TransferFuture<'a>;

    /// Future returned by the `transfer_in_place` method.
    type TransferInPlaceFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Write and read simultaneously. The contents of `words` are
    /// written to the slave, and the received words are stored into the same
    /// `words` buffer, overwriting it.
    fn transfer_in_place<'a>(
        &'a mut self,
        words: &'a mut [Word],
    ) -> Self::TransferInPlaceFuture<'a>;

    /// Future returned by the `transaction` method.
    type TransactionFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Execute multiple operations as part of a single SPI transaction
    fn transaction<'a>(
        &'a mut self,
        operations: &'a mut [Operation<'a, Word>],
    ) -> Self::TransactionFuture<'a>;
}

impl<T: ReadWrite<Word>, Word: 'static + Copy> ReadWrite<Word> for &mut T {
    type TransferFuture<'a>
    where
        Self: 'a,
    = T::TransferFuture<'a>;

    fn transfer<'a>(
        &'a mut self,
        read: &'a mut [Word],
        write: &'a [Word],
    ) -> Self::TransferFuture<'a> {
        T::transfer(self, read, write)
    }

    type TransferInPlaceFuture<'a>
    where
        Self: 'a,
    = T::TransferInPlaceFuture<'a>;

    fn transfer_in_place<'a>(
        &'a mut self,
        words: &'a mut [Word],
    ) -> Self::TransferInPlaceFuture<'a> {
        T::transfer_in_place(self, words)
    }

    type TransactionFuture<'a>
    where
        Self: 'a,
    = T::TransactionFuture<'a>;

    fn transaction<'a>(
        &'a mut self,
        operations: &'a mut [Operation<'a, Word>],
    ) -> Self::TransactionFuture<'a> {
        T::transaction(self, operations)
    }
}
