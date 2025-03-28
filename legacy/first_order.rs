    pub fn generate_first_order_duplication(&self, candidate: Vec<Key>) -> Vec<Vec<(u64, u64)>> {
        type Hasher = [Element; 4];
        let mut res_map: FxHashMap<Hasher, FxHashMap<Element, u64>> = FxHashMap::default();
        for encodable in &self.config.encodables {
            let sequence = encodable.sequence.clone();
            for index in 0..sequence.len() {
                let mut tuple: [Element; 4] = [0; 4];
                for (i, ptr) in tuple.iter_mut().enumerate() {
                    *ptr = sequence[i];
                }
                tuple[index] = 0;
                res_map
                    .entry(tuple)
                    .and_modify(|m| {
                        if !m.contains_key(&sequence[index]) {
                            m.insert(sequence[index], encodable.frequency);
                        }
                    })
                    .or_insert_with(|| {
                        let mut map = FxHashMap::default();
                        map.insert(sequence[index], encodable.frequency);
                        map
                    });
            }
        }
        let mut result = vec![vec![(0, 0); candidate.len()]; candidate.len()];
        for (_key, value) in res_map.iter() {
            for (i, (ei, fi)) in value.iter().enumerate() {
                for (j, (ej, fj)) in value.iter().enumerate() {
                    if i >= j {
                        continue;
                    }
                    result[*ei][*ej].0 += 1;
                    result[*ei][*ej].1 += fi.min(fj);
                }
            }
        }
        result
    }