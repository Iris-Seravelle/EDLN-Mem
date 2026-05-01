use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct TelemetryTracker {
    pub total_faults: usize,
    pub total_latency: Duration,
    latency_stats: Vec<Duration>,
    heat_map: HashMap<usize, usize>,
    last_report: Instant,
}

impl TelemetryTracker {
    pub fn new() -> Self {
        TelemetryTracker {
            total_faults: 0,
            total_latency: Duration::from_secs(0),
            latency_stats: Vec::new(),
            heat_map: HashMap::new(),
            last_report: Instant::now(),
        }
    }

    pub fn record_fault(&mut self, addr: usize, duration: Duration) {
        self.total_faults += 1;
        self.total_latency += duration;
        self.latency_stats.push(duration);
        *self.heat_map.entry(addr).or_insert(0) += 1;

        // Print a report every 1 second if there's activity
        if self.last_report.elapsed() > Duration::from_secs(1) {
            self.report();
            self.last_report = Instant::now();
        }
    }

    pub fn report(&self) {
        if self.latency_stats.is_empty() {
            return;
        }

        let total_duration: Duration = self.latency_stats.iter().sum();
        let avg_latency = total_duration / self.latency_stats.len() as u32;
        let max_latency = self
            .latency_stats
            .iter()
            .max()
            .cloned()
            .unwrap_or(Duration::from_secs(0));

        println!("\n--- [Telemetry Report] ---");
        println!("Total Faults: {}", self.latency_stats.len());
        println!("Avg Latency:  {:?}", avg_latency);
        println!("Max Latency:  {:?}", max_latency);
        println!("Hot Pages (top 5):");

        let mut hot_pages: Vec<(&usize, &usize)> = self.heat_map.iter().collect();
        hot_pages.sort_by(|a, b| b.1.cmp(a.1));

        for (addr, count) in hot_pages.iter().take(5) {
            println!("  0x{:x}: {} accesses", addr, count);
        }
        println!("--------------------------\n");
    }
}
