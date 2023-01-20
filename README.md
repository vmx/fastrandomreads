Fast random reads
=================

Create random offsets file:

```console
git clone https://github.com/filecoin-project/rust-fil-proofs
cd rust-fil-proofs
mkidr /tmp/parentcache
FIL_PROOFS_PARENT_CACHE=/tmp/parentcache cargo run --release --bin gen_graph_cache -- --size $(bc <<< '32 * 1024^3')
```

License
-------

Dual licensed under MIT or Apache License (Version 2.0). See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE) for more details.
