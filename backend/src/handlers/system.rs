use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct SystemStats {
    pub cpu_count: usize,
    pub memory_total_kb: u64,
    pub memory_available_kb: u64,
    pub memory_used_percent: f64,
    pub disk_total_bytes: u64,
    pub disk_available_bytes: u64,
    pub disk_used_percent: f64,
    pub uptime_seconds: u64,
    pub load_average: LoadAverage,
}

#[derive(Serialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

fn parse_meminfo() -> (u64, u64) {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total = 0u64;
    let mut available = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            available = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }

    (total, available)
}

fn parse_loadavg() -> LoadAverage {
    let content = fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let parts: Vec<&str> = content.split_whitespace().collect();

    LoadAverage {
        one: parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        five: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        fifteen: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0),
    }
}

fn parse_uptime() -> u64 {
    let content = fs::read_to_string("/proc/uptime").unwrap_or_default();
    content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|f| f as u64)
        .unwrap_or(0)
}

fn get_disk_stats(path: &str) -> (u64, u64) {
    // Use statvfs via libc
    use std::ffi::CString;
    let c_path = CString::new(path).unwrap_or_default();

    #[cfg(target_os = "linux")]
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
            let total = stat.f_blocks * stat.f_frsize;
            let available = stat.f_bavail * stat.f_frsize;
            return (total, available);
        }
    }

    (0, 0)
}

async fn system_stats() -> Json<SystemStats> {
    let cpu_count = num_cpus();
    let (mem_total, mem_available) = parse_meminfo();
    let mem_used_percent = if mem_total > 0 {
        ((mem_total - mem_available) as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let (disk_total, disk_available) = get_disk_stats("/");
    let disk_used_percent = if disk_total > 0 {
        ((disk_total - disk_available) as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    Json(SystemStats {
        cpu_count,
        memory_total_kb: mem_total,
        memory_available_kb: mem_available,
        memory_used_percent: mem_used_percent,
        disk_total_bytes: disk_total,
        disk_available_bytes: disk_available,
        disk_used_percent,
        uptime_seconds: parse_uptime(),
        load_average: parse_loadavg(),
    })
}

fn num_cpus() -> usize {
    let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    content
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count()
        .max(1)
}

pub fn system_routes() -> Router {
    Router::new().route("/stats", get(system_stats))
}
