#[derive(Debug, Default)]
pub struct MemStats {
    hits: usize,
    misses: usize,
}

impl MemStats {
    pub fn new() -> Self {
        MemStats {
            hits: 0,
            misses: 0,
        }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn record_miss_read(&mut self) {
        self.hits -= 1;
    }
    

    pub fn total_accesses(&self) -> usize {
        self.hits + self.misses
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total_accesses();
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }


    pub fn print_summary(&self) {
        println!("\tAccesses: {}", self.total_accesses());
        println!("\tHits: {}", self.hits);
        println!("\tMisses: {}", self.misses);
        println!("\tHit Rate: {:.2}%", self.hit_rate() * 100.0);
        println!("\tMiss Rate: {:.2}%", self.miss_rate() * 100.0);
    }
}
