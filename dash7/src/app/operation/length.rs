
use deku::{ctx::{BitSize, Endian}, no_std_io, prelude::*};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, Copy)]
pub struct Length(
    #[deku(
        reader = "Length::read(deku::reader)",
        writer = "Length::write(deku::writer, &self.0)"
    )]
    pub(crate) u32,
);

impl Into<u32> for Length {
    fn into(self) -> u32 {
        self.0 as u32
    }
}

impl From<u32> for Length {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Into<usize> for Length {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for Length {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Length {
    fn required_bits(value: u32) -> u32 {
        // This may be slow. There are faster ways, but we're not optimising for speed anyway
        value.checked_ilog2().unwrap_or(0) + 1
    }

    fn read<R>(reader: &mut Reader<R>) -> Result<u32, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek
    {
        let size =
            <u8 as DekuReader<'_, _>>::from_reader_with_ctx(reader, (Endian::Big, BitSize(2)))?;
        let value = <u32 as DekuReader<'_, _>>::from_reader_with_ctx(
            reader,
            (Endian::Big, BitSize((6 + (size * u8::BITS as u8)) as usize)),
        )?;
        Ok(value)
    }

    fn write<W>(output: &mut Writer<W>, value: &u32) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        let num_extra_bits = Length::required_bits(*value).checked_sub(6).unwrap_or(0);

        let mut num_extra_bytes = num_extra_bits.checked_div(u8::BITS).unwrap_or(0);
        if (num_extra_bits % u8::BITS) > 0 {
            num_extra_bytes += 1;
        }

        DekuWriter::to_writer(&num_extra_bytes, output, (Endian::Big, BitSize(2)))?;
        DekuWriter::to_writer(
            value,
            output,
            (
                Endian::Big,
                BitSize((6 + num_extra_bytes * u8::BITS) as usize),
            ),
        )?;

        Ok(())
    }
}
