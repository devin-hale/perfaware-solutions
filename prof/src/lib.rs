use std::{
    arch::asm,
    collections::HashMap,
    fmt::Display,
    sync::{LazyLock, RwLock},
    time::Duration,
};

pub static CTX: LazyLock<RwLock<Context>> = LazyLock::new(|| RwLock::new(Context::new()));
static TSC_FREQ: LazyLock<f64> = LazyLock::new(|| tsc_freq() as f64);
static TOTAL_ELAPSED: RwLock<Duration> = RwLock::new(Duration::ZERO);

pub struct Context {
    labels: HashMap<String, usize>,
    anchors: Vec<Anchor>,
    next_idx: usize,
    parent: Option<usize>,
}

struct Anchor {
    label: String,
    elapsed: Duration,
    children_elapsed: Duration,
}

impl Anchor {
    fn new(label: &str) -> Self {
        Self {
            label: String::from(label),
            elapsed: Duration::ZERO,
            children_elapsed: Duration::ZERO,
        }
    }

    fn elapsed(&self) -> Duration {
        self.elapsed - self.children_elapsed
    }

    fn percent_of_total(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        let total = TOTAL_ELAPSED.read().unwrap().as_secs_f64();
        (elapsed / total) * 100.0
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{: <20} {: <10?} ({:.2?}%)",
            self.label,
            self.elapsed(),
            self.percent_of_total(),
        )
    }
}

#[derive(Debug)]
pub struct AnchorPoint {
    idx: usize,
    parent: Option<usize>,
    label: String,
    start: u64,
}

impl AnchorPoint {
    pub fn new(label: &str) -> AnchorPoint {
        CTX.write().unwrap().anchor_point(label)
    }

    fn dur(&self) -> Duration {
        let cycles = tsc() - self.start;
        Duration::from_secs_f64(cycles as f64 / TSC_FREQ.clone())
    }
}

impl Drop for AnchorPoint {
    fn drop(&mut self) {
        CTX.write().unwrap().drop_anchor_point(self);
    }
}

impl Context {
    fn new() -> Self {
        Self {
            labels: HashMap::new(),
            next_idx: 0,
            anchors: vec![],
            parent: None,
        }
    }

    fn drop_anchor_point(&mut self, ap: &AnchorPoint) {
        let dur = ap.dur();
        self.anchors[ap.idx].elapsed += dur;
        if let Some(p) = ap.parent {
            self.anchors[p].children_elapsed += dur;
            self.parent = Some(p);
        }
        if ap.idx == 0 {
            *TOTAL_ELAPSED.write().unwrap() += dur;
            let total = TOTAL_ELAPSED.read().unwrap();
            println!("Total Time {:?}", total);
            println!("{:_<40}", "");
            self.anchors
                .sort_by(|a, b| b.percent_of_total().total_cmp(&a.percent_of_total()));
            for (_, s) in self.anchors.iter().enumerate() {
                println!("{s}");
            }
        }
    }

    fn anchor_point(&mut self, label: &str) -> AnchorPoint {
        if let Some(idx) = self.labels.get(label) {
            let idx = *idx;
            let parent = self.parent;
            self.parent = Some(idx);
            AnchorPoint {
                parent,
                idx,
                label: String::from(label),
                start: tsc(),
            }
        } else {
            let idx = self.next_idx;
            self.labels.insert(String::from(label), idx);
            self.anchors.push(Anchor::new(label));
            self.next_idx += 1;

            let parent = self.parent;
            self.parent = Some(idx);
            AnchorPoint {
                parent,
                idx,
                label: String::from(label),
                start: tsc(),
            }
        }
    }
}

pub fn tsc_freq() -> u64 {
    let freq: u64;
    unsafe {
        asm!( "mrs {out}, cntfrq_el0", out = out(reg) freq);
    }
    freq
}

pub fn tsc() -> u64 {
    let tsc: u64;
    unsafe {
        asm!( "mrs {out}, cntvct_el0", out = out(reg) tsc);
    }
    tsc
}
