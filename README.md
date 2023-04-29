# Auth Encrypt (take two)

This is a rewrite of auth-encrypt to use hyper directly and to do all cryptography in-process.

## Usage

Set the `LISTEN_ON` env var to the Unix socket you want the server to listen on. The server will serve from its working directory.

It will prevent escapes like `/../../etc/passwd` but will allow symlinks to escape.

## License

0BSD
