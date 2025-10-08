fetch:
	mkdir -p assets; \
	for file in key_distribution.txt pair_equivalence.txt; do \
		curl "https://assets.chaifen.app/$$file" -o assets/$$file; \
	done

PARAMS = data/config.yaml -e data/elements.txt -k data/dist.txt -p data/linear_multiple.txt

e:
	cargo run --release -- $(PARAMS) encode

p:
	cargo run --release -- $(PARAMS) -t 10 optimize

s:
	cargo run --release -- $(PARAMS) optimize

i:
	cargo instruments --profile bench -t time -- $(PARAMS) -t 8 optimize
