use crate::{FixedSizeValue, FromHciBytes, FromHciBytesError, WriteHci};

unsafe impl FixedSizeValue for () {
    #[inline(always)]
    fn is_valid(_data: &[u8]) -> bool {
        true
    }
}

unsafe impl FixedSizeValue for bool {
    #[inline(always)]
    fn is_valid(data: &[u8]) -> bool {
        data.len() == 1 && data[0] < 2
    }
}

impl<'a> WriteHci for &'a [u8] {
    #[inline(always)]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn write_hci<W: embedded_io::Write>(&self, mut writer: W) -> Result<(), W::Error> {
        writer.write_all(&[self.size() as u8])?;
        writer.write_all(self)
    }

    #[inline(always)]
    async fn write_hci_async<W: embedded_io_async::Write>(&self, mut writer: W) -> Result<(), W::Error> {
        writer.write_all(&[self.size() as u8]).await?;
        writer.write_all(self).await
    }
}

impl<'de> FromHciBytes<'de> for &'de [u8] {
    fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), FromHciBytesError> {
        match data.split_first() {
            Some((&len, data)) => {
                let len = usize::from(len);
                if data.len() >= len {
                    Ok(data.split_at(len))
                } else {
                    Err(FromHciBytesError::InvalidSize)
                }
            }
            None => Err(FromHciBytesError::InvalidSize),
        }
    }
}

unsafe impl<const N: usize> FixedSizeValue for [u8; N] {
    #[inline(always)]
    fn is_valid(_data: &[u8]) -> bool {
        true
    }
}

impl<T: WriteHci> WriteHci for Option<T> {
    #[inline(always)]
    fn size(&self) -> usize {
        self.as_ref().map(|x| x.size()).unwrap_or_default()
    }

    #[inline(always)]
    fn write_hci<W: embedded_io::Write>(&self, writer: W) -> Result<(), W::Error> {
        match self {
            Some(val) => val.write_hci(writer),
            None => Ok(()),
        }
    }

    #[inline(always)]
    async fn write_hci_async<W: embedded_io_async::Write>(&self, writer: W) -> Result<(), W::Error> {
        match self {
            Some(val) => val.write_hci_async(writer).await,
            None => Ok(()),
        }
    }
}

impl<'de, T: FromHciBytes<'de>> FromHciBytes<'de> for Option<T> {
    fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), FromHciBytesError> {
        if data.is_empty() {
            Ok((None, data))
        } else {
            T::from_hci_bytes(data).map(|(x, y)| (Some(x), y))
        }
    }
}
