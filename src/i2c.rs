//! Blocking I2C API
//!
//! This API supports 7-bit and 10-bit addresses. Traits feature an `AddressMode`
//! marker type parameter. Two implementation of the `AddressMode` exist:
//! `SevenBitAddress` and `TenBitAddress`.
//!
//! Through this marker types it is possible to implement each address mode for
//! the traits independently in `embedded-hal` implementations and device drivers
//! can depend only on the mode that they support.
//!
//! Additionally, the I2C 10-bit address mode has been developed to be fully
//! backwards compatible with the 7-bit address mode. This allows for a
//! software-emulated 10-bit addressing implementation if the address mode
//! is not supported by the hardware.
//!
//! Since 7-bit addressing is the mode of the majority of I2C devices,
//! `SevenBitAddress` has been set as default mode and thus can be omitted if desired.
//!
//! ## Examples
//!
//! ### `embedded-hal` implementation for an MCU
//! Here is an example of an embedded-hal implementation of the `Write` trait
//! for both modes:
//! ```
//! # use embedded_hal::i2c::{ErrorKind, SevenBitAddress, TenBitAddress, blocking::Write};
//! /// I2C0 hardware peripheral which supports both 7-bit and 10-bit addressing.
//! pub struct I2c0;
//!
//! impl Write<SevenBitAddress> for I2c0
//! {
//! #   type Error = ErrorKind;
//! #
//!     fn write(&mut self, addr: u8, output: &[u8]) -> Result<(), Self::Error> {
//!         // ...
//! #       Ok(())
//!     }
//! }
//!
//! impl Write<TenBitAddress> for I2c0
//! {
//! #   type Error = ErrorKind;
//! #
//!     fn write(&mut self, addr: u16, output: &[u8]) -> Result<(), Self::Error> {
//!         // ...
//! #       Ok(())
//!     }
//! }
//! ```
//!
//! ### Device driver compatible only with 7-bit addresses
//!
//! For demonstration purposes the address mode parameter has been omitted in this example.
//!
//! ```
//! # use embedded_hal::i2c::{blocking::WriteRead, Error};
//! const ADDR: u8  = 0x15;
//! # const TEMP_REGISTER: u8 = 0x1;
//! pub struct TemperatureSensorDriver<I2C> {
//!     i2c: I2C,
//! }
//!
//! impl<I2C, E: Error> TemperatureSensorDriver<I2C>
//! where
//!     I2C: WriteRead<Error = E>,
//! {
//!     pub fn read_temperature(&mut self) -> Result<u8, E> {
//!         let mut temp = [0];
//!         self.i2c
//!             .write_read(ADDR, &[TEMP_REGISTER], &mut temp)
//!             .and(Ok(temp[0]))
//!     }
//! }
//! ```
//!
//! ### Device driver compatible only with 10-bit addresses
//!
//! ```
//! # use embedded_hal::i2c::{Error, TenBitAddress, blocking::WriteRead};
//! const ADDR: u16  = 0x158;
//! # const TEMP_REGISTER: u8 = 0x1;
//! pub struct TemperatureSensorDriver<I2C> {
//!     i2c: I2C,
//! }
//!
//! impl<I2C, E: Error> TemperatureSensorDriver<I2C>
//! where
//!     I2C: WriteRead<TenBitAddress, Error = E>,
//! {
//!     pub fn read_temperature(&mut self) -> Result<u8, E> {
//!         let mut temp = [0];
//!         self.i2c
//!             .write_read(ADDR, &[TEMP_REGISTER], &mut temp)
//!             .and(Ok(temp[0]))
//!     }
//! }
//! ```

use crate::private;

/// I2C error
pub trait Error: core::fmt::Debug {
    /// Convert error to a generic I2C error kind
    ///
    /// By using this method, I2C errors freely defined by HAL implementations
    /// can be converted to a set of generic I2C errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

/// I2C error kind
///
/// This represents a common set of I2C operation errors. HAL implementations are
/// free to define more specific or additional error types. However, by providing
/// a mapping to these common I2C errors, generic code can still react to them.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Bus error occurred. e.g. A START or a STOP condition is detected and is not
    /// located after a multiple of 9 SCL clock pulses.
    Bus,
    /// The arbitration was lost, e.g. electrical problems with the clock signal
    ArbitrationLoss,
    /// A bus operation was not acknowledged, e.g. due to the addressed device not
    /// being available on the bus or the device not being ready to process requests
    /// at the moment
    NoAcknowledge(NoAcknowledgeSource),
    /// The peripheral receive buffer was overrun
    Overrun,
    /// A different error occurred. The original error may contain more information.
    Other,
}

/// I2C no acknowledge error source
///
/// In cases where it is possible, a device should indicate if a no acknowledge
/// response was received to an address versus a no acknowledge to a data byte.
/// Where it is not possible to differentiate, `Unknown` should be indicated.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NoAcknowledgeSource {
    /// The device did not acknowledge its address. The device may be missing.
    Address,
    /// The device did not acknowledge the data. It may not be ready to process
    /// requests at the moment.
    Data,
    /// Either the device did not acknowledge its address or the data, but it is
    /// unknown which.
    Unknown,
}

impl Error for ErrorKind {
    fn kind(&self) -> ErrorKind {
        *self
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bus => write!(f, "Bus error occurred"),
            Self::ArbitrationLoss => write!(f, "The arbitration was lost"),
            Self::NoAcknowledge(s) => s.fmt(f),
            Self::Overrun => write!(f, "The peripheral receive buffer was overrun"),
            Self::Other => write!(
                f,
                "A different error occurred. The original error may contain more information"
            ),
        }
    }
}

impl core::fmt::Display for NoAcknowledgeSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Address => write!(f, "The device did not acknowledge its address"),
            Self::Data => write!(f, "The device did not acknowledge the data"),
            Self::Unknown => write!(f, "The device did not acknowledge its address or the data"),
        }
    }
}

/// Address mode (7-bit / 10-bit)
///
/// Note: This trait is sealed and should not be implemented outside of this crate.
pub trait AddressMode: private::Sealed {}

/// 7-bit address mode type
pub type SevenBitAddress = u8;

/// 10-bit address mode type
pub type TenBitAddress = u16;

impl AddressMode for SevenBitAddress {}

impl AddressMode for TenBitAddress {}

/// Blocking I2C traits
pub mod blocking {

    use super::{AddressMode, Error, SevenBitAddress};

    /// Blocking read
    pub trait Read<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Reads enough bytes from slave with `address` to fill `buffer`
        ///
        /// # I2C Events (contract)
        ///
        /// ``` text
        /// Master: ST SAD+R        MAK    MAK ...    NMAK SP
        /// Slave:           SAK B0     B1     ... BN
        /// ```
        ///
        /// Where
        ///
        /// - `ST` = start condition
        /// - `SAD+R` = slave address followed by bit 1 to indicate reading
        /// - `SAK` = slave acknowledge
        /// - `Bi` = ith byte of data
        /// - `MAK` = master acknowledge
        /// - `NMAK` = master no acknowledge
        /// - `SP` = stop condition
        fn read(&mut self, address: A, buffer: &mut [u8]) -> Result<(), Self::Error>;
    }

    impl<A: AddressMode, T: Read<A>> Read<A> for &mut T {
        type Error = T::Error;

        fn read(&mut self, address: A, buffer: &mut [u8]) -> Result<(), Self::Error> {
            T::read(self, address, buffer)
        }
    }

    /// Blocking write
    pub trait Write<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Writes bytes to slave with address `address`
        ///
        /// # I2C Events (contract)
        ///
        /// ``` text
        /// Master: ST SAD+W     B0     B1     ... BN     SP
        /// Slave:           SAK    SAK    SAK ...    SAK
        /// ```
        ///
        /// Where
        ///
        /// - `ST` = start condition
        /// - `SAD+W` = slave address followed by bit 0 to indicate writing
        /// - `SAK` = slave acknowledge
        /// - `Bi` = ith byte of data
        /// - `SP` = stop condition
        fn write(&mut self, address: A, bytes: &[u8]) -> Result<(), Self::Error>;
    }

    impl<A: AddressMode, T: Write<A>> Write<A> for &mut T {
        type Error = T::Error;

        fn write(&mut self, address: A, bytes: &[u8]) -> Result<(), Self::Error> {
            T::write(self, address, bytes)
        }
    }

    /// Blocking write (iterator version)
    pub trait WriteIter<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Writes bytes to slave with address `address`
        ///
        /// # I2C Events (contract)
        ///
        /// Same as `Write`
        fn write_iter<B>(&mut self, address: A, bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>;
    }

    impl<A: AddressMode, T: WriteIter<A>> WriteIter<A> for &mut T {
        type Error = T::Error;

        fn write_iter<B>(&mut self, address: A, bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            T::write_iter(self, address, bytes)
        }
    }

    /// Blocking write + read
    pub trait WriteRead<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Writes bytes to slave with address `address` and then reads enough bytes to fill `buffer` *in a
        /// single transaction*
        ///
        /// # I2C Events (contract)
        ///
        /// ``` text
        /// Master: ST SAD+W     O0     O1     ... OM     SR SAD+R        MAK    MAK ...    NMAK SP
        /// Slave:           SAK    SAK    SAK ...    SAK          SAK I0     I1     ... IN
        /// ```
        ///
        /// Where
        ///
        /// - `ST` = start condition
        /// - `SAD+W` = slave address followed by bit 0 to indicate writing
        /// - `SAK` = slave acknowledge
        /// - `Oi` = ith outgoing byte of data
        /// - `SR` = repeated start condition
        /// - `SAD+R` = slave address followed by bit 1 to indicate reading
        /// - `Ii` = ith incoming byte of data
        /// - `MAK` = master acknowledge
        /// - `NMAK` = master no acknowledge
        /// - `SP` = stop condition
        fn write_read(
            &mut self,
            address: A,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error>;
    }

    impl<A: AddressMode, T: WriteRead<A>> WriteRead<A> for &mut T {
        type Error = T::Error;

        fn write_read(
            &mut self,
            address: A,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            T::write_read(self, address, bytes, buffer)
        }
    }

    /// Blocking write (iterator version) + read
    pub trait WriteIterRead<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Writes bytes to slave with address `address` and then reads enough bytes to fill `buffer` *in a
        /// single transaction*
        ///
        /// # I2C Events (contract)
        ///
        /// Same as the `WriteRead` trait
        fn write_iter_read<B>(
            &mut self,
            address: A,
            bytes: B,
            buffer: &mut [u8],
        ) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>;
    }

    impl<A: AddressMode, T: WriteIterRead<A>> WriteIterRead<A> for &mut T {
        type Error = T::Error;

        fn write_iter_read<B>(
            &mut self,
            address: A,
            bytes: B,
            buffer: &mut [u8],
        ) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            T::write_iter_read(self, address, bytes, buffer)
        }
    }

    /// Transactional I2C operation.
    ///
    /// Several operations can be combined as part of a transaction.
    #[derive(Debug, PartialEq)]
    pub enum Operation<'a> {
        /// Read data into the provided buffer
        Read(&'a mut [u8]),
        /// Write data from the provided buffer
        Write(&'a [u8]),
    }

    /// Transactional I2C interface.
    ///
    /// This allows combining operations within an I2C transaction.
    pub trait Transactional<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Execute the provided operations on the I2C bus.
        ///
        /// Transaction contract:
        /// - Before executing the first operation an ST is sent automatically. This is followed by SAD+R/W as appropriate.
        /// - Data from adjacent operations of the same type are sent after each other without an SP or SR.
        /// - Between adjacent operations of a different type an SR and SAD+R/W is sent.
        /// - After executing the last operation an SP is sent automatically.
        /// - If the last operation is a `Read` the master does not send an acknowledge for the last byte.
        ///
        /// - `ST` = start condition
        /// - `SAD+R/W` = slave address followed by bit 1 to indicate reading or 0 to indicate writing
        /// - `SR` = repeated start condition
        /// - `SP` = stop condition
        fn exec<'a>(
            &mut self,
            address: A,
            operations: &mut [Operation<'a>],
        ) -> Result<(), Self::Error>;
    }

    impl<A: AddressMode, T: Transactional<A>> Transactional<A> for &mut T {
        type Error = T::Error;

        fn exec<'a>(
            &mut self,
            address: A,
            operations: &mut [Operation<'a>],
        ) -> Result<(), Self::Error> {
            T::exec(self, address, operations)
        }
    }

    /// Transactional I2C interface (iterator version).
    ///
    /// This allows combining operation within an I2C transaction.
    pub trait TransactionalIter<A: AddressMode = SevenBitAddress> {
        /// Error type
        type Error: Error;

        /// Execute the provided operations on the I2C bus (iterator version).
        ///
        /// Transaction contract:
        /// - Before executing the first operation an ST is sent automatically. This is followed by SAD+R/W as appropriate.
        /// - Data from adjacent operations of the same type are sent after each other without an SP or SR.
        /// - Between adjacent operations of a different type an SR and SAD+R/W is sent.
        /// - After executing the last operation an SP is sent automatically.
        /// - If the last operation is a `Read` the master does not send an acknowledge for the last byte.
        ///
        /// - `ST` = start condition
        /// - `SAD+R/W` = slave address followed by bit 1 to indicate reading or 0 to indicate writing
        /// - `SR` = repeated start condition
        /// - `SP` = stop condition
        fn exec_iter<'a, O>(&mut self, address: A, operations: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = Operation<'a>>;
    }

    impl<A: AddressMode, T: TransactionalIter<A>> TransactionalIter<A> for &mut T {
        type Error = T::Error;

        fn exec_iter<'a, O>(&mut self, address: A, operations: O) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = Operation<'a>>,
        {
            T::exec_iter(self, address, operations)
        }
    }
}
