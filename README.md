# chat-and-share-fs

Early steps towards a HTTP server that supports a chat-log and interacting with the local filesystem.

For now lets you browse the filesystem by generating an `index.html` file on the fly when you `GET` a folder.

Start the server on [localhost:8080](http://localhost:8080/) with 
```bash
cargo run -- --root .
```

Adapted from [this writeup](https://dev.to/geoffreycopin/build-a-http-server-with-rust-and-tokio-part-1-serving-static-files-165l) (thank you Geoffrey!).