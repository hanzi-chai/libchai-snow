fetch:
	mkdir -p assets; \
	for file in key_distribution.txt pair_equivalence.txt; do \
		curl "https://assets.chaifen.app/$$file" -o assets/$$file; \
	done

COMMON = -k assets/key_distribution.txt -p assets/pair_equivalence.txt

FH_PARAMS = project-feihua/config.yaml -e project-feihua/elements.yaml $(COMMON)

fed:
	cargo run --bin feihua -- encode $(FH_PARAMS)

fe:
	cargo run --release --bin feihua -- encode $(FH_PARAMS)

fo:
	cargo run --release --bin feihua -- optimize $(FH_PARAMS)

fp:
	cargo run --release --bin feihua -- optimize $(FH_PARAMS) -t 10

S2_PARAMS = project-snow2/config.yaml -e project-snow2/elements.txt $(COMMON)

s2d:
	cargo run --bin snow2 -- encode $(S2_PARAMS)

s2e:
	cargo run --release --bin snow2 -- encode $(S2_PARAMS)

s2o:
	cargo run --release --bin snow2 -- optimize $(S2_PARAMS)

s2p:
	cargo run --release --bin snow2 -- optimize $(S2_PARAMS) -t 10

QY_PARAMS = project-qingyun/config.yaml -e project-qingyun/elements.txt $(COMMON)

qyd:
	cargo run --bin qingyun -- encode $(QY_PARAMS)

qye:
	cargo run --release --bin qingyun -- encode $(QY_PARAMS)

qyo:
	cargo run --release --bin qingyun -- optimize $(QY_PARAMS)

qyp:
	cargo run --release --bin qingyun -- optimize $(QY_PARAMS) -t 10