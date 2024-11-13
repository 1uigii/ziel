use tokio::io;

pub mod client;
pub mod server;

pub(crate) mod raw;

#[derive(thiserror::Error, Debug)]
pub enum Error<M>
where
    M: raw::TryFromMessage,
{
    #[error("protocol :: io :: {0}")]
    Io(#[from] io::Error),
    #[error("protocol :: {0}")]
    Protocol(M::Error),
}

impl<M: raw::TryFromMessage> Error<M> {
    fn from_prot_err(err: M::Error) -> Error<M> {
        Error::<M>::Protocol(err)
    }
}

pub async fn write<R, M>(writer: &mut R, message: M) -> Result<(), io::Error>
where
    R: io::AsyncWriteExt + std::marker::Unpin,
    M: raw::IntoMessage,
{
    let message: raw::Message = message.into_raw_message();
    writer.write_u8(message.type_marker).await?;
    writer.write_u32(message.body.len() as u32).await?;
    writer.write_all(&message.body).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read<R, M>(reader: &mut R) -> Result<M, Error<M>>
where
    R: io::AsyncReadExt + std::marker::Unpin,
    M: raw::TryFromMessage,
{
    let type_marker = reader.read_u8().await?;
    let length_marker = reader.read_u32().await?;
    let mut body = vec![0; length_marker as usize];
    reader.read_exact(&mut body).await?;

    let message = raw::Message { type_marker, body };

    M::try_from_raw_message(message).map_err(Error::<M>::from_prot_err)
}

// TODO
#[cfg(test)]
mod tests {
    use super::*;
}
