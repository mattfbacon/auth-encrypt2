use std::io::{self};
use std::pin::Pin;
use std::task::{ready, Context, Poll};

use bytes::Bytes;
use chacha20::cipher::{KeyIvInit as _, StreamCipher as _, StreamCipherSeekCore as _};
use chacha20::{ChaCha20, ChaChaCore};
use futures_core::Stream;
use hmac::Hmac;
use sha2::Sha256;
use tokio::io::{AsyncRead, AsyncReadExt as _, ReadBuf};
use tokio_util::io::ReaderStream;

struct CipherWrapper<S> {
	cipher: ChaCha20,
	inner: S,
}

impl<S: AsyncRead + Unpin> AsyncRead for CipherWrapper<S> {
	fn poll_read(
		self: Pin<&mut Self>,
		ctx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let this = self.get_mut();
		let old_filled_len = buf.filled().len();
		ready!(Pin::new(&mut this.inner).poll_read(ctx, buf)?);
		let read = &mut buf.filled_mut()[old_filled_len..];
		this.cipher.apply_keystream(read);
		Poll::Ready(Ok(()))
	}
}

pub async fn decrypt(
	mut file: impl tokio::io::AsyncBufRead + Unpin,
	password: Bytes,
) -> io::Result<impl Stream<Item = io::Result<Bytes>>> {
	const MAGIC_LENGTH: usize = 8;
	const MAGIC: &[u8; MAGIC_LENGTH] = b"Salted__";
	const SALT_LENGTH: usize = 8;
	const KEY_LENGTH: usize = 32;
	const COUNTER_LENGTH: usize = 4;
	const IV_LENGTH: usize = 12;
	const ROUNDS: u32 = 10_000;

	let mut magic = [0u8; MAGIC_LENGTH];
	file.read_exact(&mut magic).await?;
	if &magic != MAGIC {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "bad magic"));
	}

	let mut salt = [0u8; SALT_LENGTH];
	file.read_exact(&mut salt).await?;

	let pbkdf2 = tokio::task::spawn_blocking(move || {
		let mut pbkdf2 = [0u8; KEY_LENGTH + COUNTER_LENGTH + IV_LENGTH];
		pbkdf2::pbkdf2::<Hmac<Sha256>>(&password, &salt, ROUNDS, &mut pbkdf2).unwrap();
		pbkdf2
	})
	.await
	.map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
	let (key, counter_iv) = &pbkdf2.split_at(KEY_LENGTH);
	let (counter, iv) = counter_iv.split_at(COUNTER_LENGTH);
	let counter = u32::from_le_bytes(counter.try_into().unwrap());

	let mut core = ChaChaCore::new_from_slices(key, iv).unwrap();
	core.set_block_pos(counter);
	let cipher = ChaCha20::from_core(core);

	Ok(ReaderStream::new(CipherWrapper {
		cipher,
		inner: file,
	}))
}
