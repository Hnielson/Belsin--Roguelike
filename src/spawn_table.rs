use rltk::RandomNumberGenerator;

pub struct Entry {
    name : String,
    weight : i32
}

impl Entry {
    pub fn new<S:ToString>(name: S, weight: i32) -> Entry {
        Entry{name: name.to_string(), weight}
    }
}

#[derive(Default)]
pub struct SpawnTable {
    entries : Vec<Entry>,
    total_weight : i32
}

impl SpawnTable {
    pub fn new() -> SpawnTable {
        SpawnTable{entries: Vec::new(), total_weight: 0}
    }

    pub fn add<S:ToString>(mut self, name : S, weight: i32) -> SpawnTable {
        self.total_weight += weight;
        self.entries.push(Entry::new(name.to_string(), weight));
        self
    }

    pub fn roll(&self, rng : &mut RandomNumberGenerator) -> String {
        if self.total_weight == 0 {return "None".to_string();}
        let mut roll = rng.roll_dice(1, self.total_weight)-1;
        let mut idx : usize = 0;

        while roll > 0 {
            if roll < self.entries[idx].weight {
                return self.entries[idx].name.clone();
            }
            roll -= self.entries[idx].weight;
            idx += 1;
        }

        "None".to_string()
    }
}