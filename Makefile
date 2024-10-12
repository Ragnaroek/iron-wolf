profile:
	sudo -E cargo flamegraph --bench core_loop -- --bench

bench:
	cargo bench