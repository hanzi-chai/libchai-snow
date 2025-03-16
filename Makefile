fetch:
	mkdir -p assets; \
	for file in key_distribution.txt pair_equivalence.txt; do \
		curl "https://assets.chaifen.app/$$file" -o assets/$$file; \
	done
