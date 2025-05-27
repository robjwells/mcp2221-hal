//! embedded_hal I2C trait implementations for MCP2221.
use embedded_hal::i2c::{self, I2c, Operation, SevenBitAddress};

use super::MCP2221;
use crate::Error;
use crate::constants::MAX_I2C_TRANSFER_PLUS_1;

impl i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        use embedded_hal::i2c::NoAcknowledgeSource::Address;
        // Generally we can't get enough information from the MCP2221 to communicate the
        // embedded_hal I2C error kinds. The Microchip drivers in some cases match
        // against the "reserved" field in I2C response buffers but I'm hesistant to do
        // that because it's explicitly undocumented.
        match self {
            Error::I2cAddressNack => i2c::ErrorKind::NoAcknowledge(Address),
            _ => i2c::ErrorKind::Other,
        }
    }
}

impl i2c::ErrorType for MCP2221 {
    type Error = Error;
}

/// Helper to chunk operations based on type (enum case).
fn same_operation_type(a: &Operation, b: &Operation) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// Sum the lengths of the given optional buffer of operations.
///
/// If there are operations and the length is in bounds, returns Ok(Some(length)).
///
/// Lifetime specifier needed to tell the compiler that the borrow of the mutable
/// reference (from chunked) live as long as each other, but that they are
/// separate from the length of the borrowed mutable buffer, and the operations
/// within.
fn sum_and_check_lengths<'a>(
    maybe_ops: Option<&'a &'a mut &'_ mut [Operation<'_>]>,
) -> Result<Option<usize>, Error> {
    maybe_ops
        .map(|ops| {
            let sum: usize = ops
                .iter()
                .map(|op| match op {
                    Operation::Read(items) => items.len(),
                    Operation::Write(items) => items.len(),
                })
                .sum();
            match sum {
                MAX_I2C_TRANSFER_PLUS_1.. => Err(Error::I2cTransferTooLong),
                0 => Err(Error::I2cTransferEmpty),
                sum => Ok(sum),
            }
        })
        .transpose()
}

type MaybeOps<'a, 'b, 'c> = Option<&'a mut &'b mut [Operation<'c>]>;
type WritesReads<'a, 'b, 'c> = (MaybeOps<'a, 'b, 'c>, MaybeOps<'a, 'b, 'c>);
fn try_get_valid_operations<'a, 'b, 'c>(
    ops: &'a mut [&'b mut [Operation<'c>]],
) -> Result<WritesReads<'a, 'b, 'c>, Error> {
    match ops {
        // Three or more chunks implies read-before-write.
        [_, _, _, ..] => Err(Error::I2cUnsupportedEmbeddedHalTransaction),
        // Single read-before-write.
        [[Operation::Read(_), ..], [Operation::Write(_), ..]] => {
            Err(Error::I2cUnsupportedEmbeddedHalTransaction)
        }
        // Write-Read
        [
            writes @ [Operation::Write(_), ..],
            reads @ [Operation::Read(_), ..],
        ] => Ok((Some(writes), Some(reads))),
        // Read
        [reads @ [Operation::Read(_), ..]] => Ok((None, Some(reads))),
        // Write
        [writes @ [Operation::Write(_), ..]] => Ok((Some(writes), None)),
        _ => unreachable!(),
    }
}

impl I2c<SevenBitAddress> for MCP2221 {
    /// Execute the provided operations on the I2C bus.
    ///
    /// <div class="warning">
    ///
    /// The MCP2221 cannot fully support the contract of [`I2c::transaction`] because it
    /// does not have a HID command to perform an I2C read without issuing a final
    /// STOP condition.
    ///
    /// Transactions that place a read operation before a write will return an error.
    ///
    /// [`transaction`]: embedded_hal::i2c::I2c::transaction()
    ///
    /// With this caveat in mind, you may wish to consult the original documentation
    /// of [`I2c::transaction`].
    ///
    /// </div>
    ///
    /// The requirements of the MCP2221's HID commands mean this method is quite
    /// complex, as it must copy all data to be written from the (potentially several)
    /// write buffers into a single buffer. Likewise, the whole read transfer is
    /// buffered before being copied into the caller's (potentially several) mutable
    /// read buffers.
    ///
    /// The other methods of the [`I2c`] trait only take individual buffers and do not
    /// incur the overhead of this much more general method, and so they should be
    /// preferred where possible.
    fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        // Because the MCP2221 needs to know transfer length up-front, and because it
        // doesn't support a read without a STOP condition, we need to coalesce the
        // passed operations and check they are possible.
        let mut chunked: Vec<&mut [Operation<'_>]> =
            operations.chunk_by_mut(same_operation_type).collect();
        let (writes, reads) = try_get_valid_operations(chunked.as_mut_slice())?;

        // Check the write and read lengths first, so we don't do any writes if there is
        // a problem with length of a read to follow. These are outside the respective
        // write/read blocks because the other block needs them to know which command
        // to use.
        let write_length = sum_and_check_lengths(writes.as_ref())?;
        let read_length = sum_and_check_lengths(reads.as_ref())?;

        if let Some(writes) = writes {
            // Buffer all data to be written in our own Vec.
            let mut write_data: Vec<u8> = Vec::with_capacity(write_length.unwrap());
            for op in writes.iter() {
                let Operation::Write(buf) = op else {
                    unreachable!("Chunk checks ensure only writes here.")
                };
                write_data.extend_from_slice(buf);
            }
            // Perform write.
            if read_length.is_none() {
                // No following read, so perform a normal write with START and STOP.
                self.i2c_write(address, &write_data)?
            } else {
                // Following read, so don't issue a STOP condition after the write.
                self.i2c_write_no_stop(address, &write_data)?
            };
        }

        if let Some(reads) = reads {
            // Single buffer to hold all the read data.
            let mut our_buffer = vec![0u8; read_length.unwrap()];
            // Perform read.
            if write_length.is_none() {
                // No preceding write, so perform a read with START and STOP at the end.
                self.i2c_read(address, our_buffer.as_mut_slice())?;
            } else {
                // If there was a write beforehand, follow their No STOP with repeated START.
                self.i2c_read_repeated_start(address, our_buffer.as_mut_slice())?;
            };
            // Fill the caller's buffers one at a time from our buffer.
            let mut copied_so_far = 0;
            for op in reads.iter_mut() {
                let Operation::Read(their_buffer) = op else {
                    unreachable!("Chunk checks ensure only reads here.");
                };
                let start = copied_so_far;
                let end = start + their_buffer.len();
                their_buffer.copy_from_slice(&our_buffer[start..end]);
                copied_so_far = end;
            }
        }

        Ok(())
    }

    fn read(&mut self, address: SevenBitAddress, read: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c_read(address, read)
    }

    fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), Self::Error> {
        self.i2c_write(address, write)
    }

    fn write_read(
        &mut self,
        address: SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c_write_read(address, write, read)
    }
}

#[cfg(feature = "async")]
mod eh_async {
    use embedded_hal::i2c::{I2c as BlockingI2c, Operation};
    use embedded_hal_async::i2c::I2c as AsyncI2c;

    impl AsyncI2c for crate::MCP2221 {
        async fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            BlockingI2c::transaction(self, address, operations)
        }

        async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            BlockingI2c::read(self, address, read)
        }

        async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            BlockingI2c::write(self, address, write)
        }

        async fn write_read(
            &mut self,
            address: u8,
            write: &[u8],
            read: &mut [u8],
        ) -> Result<(), Self::Error> {
            BlockingI2c::write_read(self, address, write, read)
        }
    }
}
