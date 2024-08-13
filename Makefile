FILE = ./object_files/2048.obj

run:
	cargo run --bin lc3-vm -- -f $(FILE)

interactive:
	cargo run --bin lc3-vm -- -i

debug:
	cargo run --bin lc3-vm -- -d $(FILE)
