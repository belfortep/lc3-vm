FILE = ./object_files/2048.obj

run:
	cargo run -- -f $(FILE)

interactive:
	cargo run -- -p 

debug:
	cargo run -- -d
