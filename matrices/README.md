# Multiply matrices example

* Get and parse files from storage.
* Multiply.
* Save results to storage.

## How to build
* To build `wasm` run: `make build`
* After build `wasm` you can convert to `wat` run: `make wat`

If you plan to multiply a big matrix, will be better to increase the size of stack.
For that you can go to `.cargo` folder, edit/create `config` file and add follow lines:
```$xslt
[target.wasm32-unknown-unknown]
rustflags = [
 "-C", "link-args=-z stack-size=15000000",
]
```