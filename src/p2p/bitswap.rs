use async_trait::async_trait;
use libp2p::request_response::{self as req_res, ProtocolSupport};
use std::io;
use futures::AsyncReadExt;
use futures::AsyncWriteExt;

const PROTOCOL: &str = "/ksh/bitswap/1.0.0";

#[derive(Debug, Clone)]
pub struct BitswapRequest(pub Vec<u8>);
#[derive(Debug, Clone)]
pub struct BitswapResponse(pub Vec<u8>);

#[derive(Debug, Clone, Default)]
pub struct BitswapCodec;

#[async_trait]
impl req_res::Codec for BitswapCodec {
    type Protocol = &'static str;
    type Request = BitswapRequest;
    type Response = BitswapResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &&'static str,
        stream: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;
        Ok(BitswapRequest(data))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &&'static str,
        stream: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        if len == 0 {
            return Ok(BitswapResponse(vec![]));
        }
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;
        Ok(BitswapResponse(data))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &&'static str,
        stream: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let len = req.0.len() as u32;
        stream.write_all(&len.to_le_bytes()).await?;
        stream.write_all(&req.0).await?;
        stream.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &&'static str,
        stream: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let len = res.0.len() as u32;
        stream.write_all(&len.to_le_bytes()).await?;
        stream.write_all(&res.0).await?;
        stream.close().await?;
        Ok(())
    }
}

pub type BitswapBehaviour = req_res::Behaviour<BitswapCodec>;

pub fn new_bitswap_behaviour() -> BitswapBehaviour {
    BitswapBehaviour::new(
        std::iter::once((PROTOCOL, ProtocolSupport::Full)),
        Default::default(),
    )
}
