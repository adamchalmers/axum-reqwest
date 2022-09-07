# About

This project demonstrates and benchmarks different approaches to reading multipart forms in Axum, a Rust HTTP server library.

The binary can be run in three modes:
 - A print server, which prints out bodies.
 - A proxy server which reads multipart bodies and proxies them somewhere else, either via
   - Buffering the whole request into memory
   - Streaming the request 

# Benchmark commands

I used curl to send the Unix dictionary to the proxy server 20 times, which proxied to the print server.

## Client

```sh
time curl -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words -F upload=@/usr/share/dict/words 127.0.0.1:3000/ && kill $(pbpaste)
```

## Proxy server

```sh
cargo build --release -q && time cargo run --release 3000 3001 --streaming
```

## Print server

```sh
cargo run 3001 > /dev/null
```

# Benchmark results

|           | Time (seconds)  | RAM (mb) |
| --------- | --------------- | -------- |
| Streaming | 0.10            | 16
| Buffering | 0.32            | 128

