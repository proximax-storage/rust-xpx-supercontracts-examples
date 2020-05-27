build:
	@cargo build --target wasm32-unknown-unknown --release
	
wat:
	@wasm2wat target/wasm32-unknown-unknown/release/multiply_matrices_SC.wasm -o ~/multiply_matrices_SC.wat
