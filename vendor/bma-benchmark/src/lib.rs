#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "README.md" ) ) ]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prettytable;

pub use bma_benchmark_proc::benchmark_stage;
use colored::Colorize;
use num_format::{Locale, ToFormattedString};
use prettytable::Table;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use terminal_size::{terminal_size, Height, Width};

lazy_static! {
    pub static ref DEFAULT_BENCHMARK: Mutex<Benchmark> = Mutex::new(Benchmark::new0());
    pub static ref DEFAULT_STAGED_BENCHMARK: Mutex<StagedBenchmark> =
        Mutex::new(StagedBenchmark::new());
}

macro_rules! result_separator {
    () => {
        separator("--- Benchmark results ")
    };
}

macro_rules! format_number {
    ($n: expr) => {
        $n.to_formatted_string(&Locale::en).replace(',', "_")
    };
}

#[macro_export]
/// run a stage of staged bechmark
macro_rules! staged_benchmark {
    ($name: expr, $iterations: expr, $code: block) => {
        bma_benchmark::staged_benchmark_start!($name);
        for _iteration in 0..$iterations
            $code
        bma_benchmark::staged_benchmark_finish!($name, $iterations);
    };
}

#[macro_export]
/// run a stage of staged bechmark and check the result for each iteration
///
/// The statement MUST return true for ok and false for errors
macro_rules! staged_benchmark_check {
    ($name: expr, $iterations: expr, $code: block) => {
        let mut bma_benchmark_errors = 0;
        bma_benchmark::staged_benchmark_start!($name);
        for _iteration in 0..$iterations {
            if !$code {
                bma_benchmark_errors += 1;
            }
        }
        bma_benchmark::staged_benchmark_finish!($name, $iterations, bma_benchmark_errors);
    };
}

#[macro_export]
/// run a benchmark
macro_rules! benchmark {
    ($iterations: expr, $code: block) => {
        bma_benchmark::benchmark_start!();
        for _iteration in 0..$iterations
            $code
        bma_benchmark::benchmark_print!($iterations);
    };
}

#[macro_export]
/// run a benchmark and check the result for each iteration
///
/// The statement MUST return true for ok and false for errors
macro_rules! benchmark_check {
    ($iterations: expr, $code: block) => {
        let mut bma_benchmark_errors = 0;
        bma_benchmark::benchmark_start!();
        for _iteration in 0..$iterations {
            if !$code {
                bma_benchmark_errors += 1;
            }
        }
        bma_benchmark::benchmark_print!($iterations, bma_benchmark_errors);
    };
}

/// Start the default stared benchmark stage
#[macro_export]
macro_rules! staged_benchmark_start {
    ($name: expr) => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .start($name);
    };
}

/// Finish the default staged benchmark stage
#[macro_export]
macro_rules! staged_benchmark_finish {
    ($name: expr, $iterations: expr) => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .finish($name, $iterations, 0);
    };
    ($name: expr, $iterations: expr, $errors: expr) => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .finish($name, $iterations, $errors);
    };
}

/// Finish the default staged benchmark current (last started) stage
#[macro_export]
macro_rules! staged_benchmark_finish_current {
    ($iterations: expr) => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .finish_current($iterations, 0);
    };
    ($iterations: expr, $errors: expr) => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .finish_current($iterations, $errors);
    };
}

/// Reset the default staged benchmark
#[macro_export]
macro_rules! staged_benchmark_reset {
    () => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .reset();
    };
}

/// Print staged benchmark result
#[macro_export]
macro_rules! staged_benchmark_print {
    () => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .print();
    };
}

/// Print staged benchmark result, specifying the reference stage
#[macro_export]
macro_rules! staged_benchmark_print_for {
    ($eta: expr) => {
        bma_benchmark::DEFAULT_STAGED_BENCHMARK
            .lock()
            .unwrap()
            .print_for($eta);
    };
}

/// Start a simple benchmark
#[macro_export]
macro_rules! benchmark_start {
    () => {
        bma_benchmark::DEFAULT_BENCHMARK.lock().unwrap().reset();
    };
}

/// Finish a simple benchmark and print results
#[macro_export]
macro_rules! benchmark_print {
    ($iterations: expr) => {
        bma_benchmark::DEFAULT_BENCHMARK
            .lock()
            .unwrap()
            .print(Some($iterations), None);
    };
    ($iterations: expr, $errors: expr) => {
        bma_benchmark::DEFAULT_BENCHMARK
            .lock()
            .unwrap()
            .print(Some($iterations), Some($errors));
    };
}

#[derive(Default)]
pub struct LatencyBenchmark {
    latencies: Vec<Duration>,
    op: Option<Instant>,
}

impl LatencyBenchmark {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        self.latencies.clear();
        self.op.take();
    }
    #[inline]
    pub fn op_start(&mut self) {
        self.op.replace(Instant::now());
    }
    /// # Panics
    ///
    /// Will panic if op is not started
    #[inline]
    pub fn op_finish(&mut self) {
        self.latencies.push(self.op.take().unwrap().elapsed());
    }
    #[inline]
    pub fn push(&mut self, latency: Duration) {
        self.latencies.push(latency);
    }
    #[allow(clippy::cast_possible_truncation)]
    pub fn avg(&self) -> Duration {
        self.latencies.iter().sum::<Duration>() / self.latencies.len() as u32
    }
    pub fn min(&self) -> Duration {
        self.latencies.iter().min().copied().unwrap_or_default()
    }
    pub fn max(&self) -> Duration {
        self.latencies.iter().max().copied().unwrap_or_default()
    }
    pub fn print(&self) {
        let avg = format_number!(self.avg().as_micros()).yellow();
        let min = format_number!(self.min().as_micros()).green();
        let max = format_number!(self.max().as_micros()).red();
        println!("latency (μs) avg: {}, min: {}, max: {}", avg, min, max);
    }
}

/// Benchmark results for a simple benchmark or a stage
pub struct BenchmarkResult {
    pub elapsed: Duration,
    pub iterations: u32,
    pub errors: u32,
    pub speed: u32,
}

/// Staged benchmark
pub struct StagedBenchmark {
    benchmarks: BTreeMap<String, Benchmark>,
    current_stage: Option<String>,
}

impl Default for StagedBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl StagedBenchmark {
    pub fn new() -> Self {
        Self {
            benchmarks: BTreeMap::new(),
            current_stage: None,
        }
    }

    /// Start benchmark stage
    ///
    /// # Panics
    ///
    /// Will panic if a stage with the same name already exists
    pub fn start(&mut self, name: &str) {
        self.current_stage = Some(name.to_owned());
        println!("{}", format!("!!! stage started: {} ", name).black());
        let benchmark = Benchmark::new0();
        assert!(
            self.benchmarks.insert(name.to_owned(), benchmark).is_none(),
            "Benchmark stage {} already exists",
            name
        );
    }

    /// Finish benchmark stage
    ///
    /// # Panics
    ///
    /// Will panic if a specified stage was not started
    pub fn finish(&mut self, name: &str, iterations: u32, errors: u32) {
        let benchmark = self
            .benchmarks
            .get_mut(name)
            .unwrap_or_else(|| panic!("Benchmark stage {} not found", name));
        benchmark.finish(Some(iterations), Some(errors));
        println!(
            "{}",
            format!(
                "*** stage completed: {} ({} iters, {:.3} secs)",
                name,
                format_number!(iterations),
                benchmark.elapsed.unwrap().as_secs_f64()
            )
            .black()
        );
    }

    /// Finish current (last started) benchmark stage
    pub fn finish_current(&mut self, iterations: u32, errors: u32) {
        let current_stage = self
            .current_stage
            .take()
            .expect("No active benchmark stage");
        self.finish(&current_stage, iterations, errors);
    }

    /// Reset staged benchmark
    pub fn reset(&mut self) {
        self.benchmarks.clear();
    }

    fn _result_table_for(&self, eta: Option<&str>) -> Table {
        let mut have_errs = false;
        let mut results: Vec<(String, BenchmarkResult)> = Vec::new();
        for (stage, benchmark) in &self.benchmarks {
            let result = benchmark.result0();
            if result.errors > 0 {
                have_errs = true;
            }
            results.push((stage.clone(), result));
        }
        let mut header = vec!["stage", "iters"];
        if have_errs {
            header.extend(["succs", "errs", "err.rate"]);
        }
        header.extend(["secs", "msecs", "iters/s"]);
        let eta_speed = eta.map(|v| {
            header.push("diff.s");
            self.benchmarks.get(v).unwrap().result0().speed
        });
        let mut table = ctable(Some(header), false);
        for (stage, benchmark) in &self.benchmarks {
            let result = benchmark.result0();
            let elapsed = result.elapsed.as_secs_f64();
            let mut cells = vec![
                cell!(stage),
                cell!(format_number!(result.iterations).magenta()),
            ];
            if have_errs {
                let success = result.iterations - result.errors;
                cells.extend([
                    cell!(if success > 0 {
                        format_number!(success).green()
                    } else {
                        <_>::default()
                    }),
                    cell!(if result.errors > 0 {
                        format_number!(result.errors).red()
                    } else {
                        <_>::default()
                    }),
                    cell!(if result.errors > 0 {
                        format!(
                            "{:.2} %",
                            (f64::from(result.errors) / f64::from(result.iterations) * 100.0)
                        )
                        .red()
                    } else {
                        "".normal()
                    }),
                ]);
            }
            cells.extend([
                cell!(format!("{:.3}", elapsed).blue()),
                cell!(format!("{:.3}", elapsed * 1000.0).cyan()),
                cell!(format_number!(result.speed).yellow()),
            ]);
            if let Some(r) = eta_speed {
                if result.speed != r {
                    let diff = f64::from(result.speed) / f64::from(r);
                    if !(0.9999..=1.0001).contains(&diff) {
                        cells.push(cell!(if diff > 1.0 {
                            format!("+{:.2} %", ((diff - 1.0) * 100.0)).green()
                        } else {
                            format!("-{:.2} %", ((1.0 - diff) * 100.0)).red()
                        }));
                    }
                }
            };
            table.add_row(prettytable::Row::new(cells));
        }
        table
    }

    /// Get the result table for staged benchmark
    pub fn result_table(&self) -> Table {
        self._result_table_for(None)
    }

    /// Get the result table for staged benchmark, specifying the reference stage
    pub fn result_table_for(&self, eta: &str) -> Table {
        self._result_table_for(Some(eta))
    }

    /// Print the result table
    pub fn print(&self) {
        println!("{}", result_separator!());
        self.result_table().printstd();
    }

    /// Print the result table, specifying the reference stage
    pub fn print_for(&self, eta: &str) {
        println!("{}", result_separator!());
        self.result_table_for(eta).printstd();
    }
}

/// Simple benchmark or a stage
pub struct Benchmark {
    started: Instant,
    iterations: u32,
    set_iterations: u32,
    errors: u32,
    elapsed: Option<Duration>,
}

impl Default for Benchmark {
    fn default() -> Self {
        Self::new0()
    }
}

impl fmt::Display for Benchmark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_string_for(Some(self.iterations), Some(self.errors))
        )
    }
}

impl Benchmark {
    /// Create simple benchmark with unknown number of iterations
    pub fn new0() -> Self {
        Self {
            started: Instant::now(),
            iterations: 0,
            set_iterations: 0,
            errors: 0,
            elapsed: None,
        }
    }

    /// Create simple benchmark with pre-defined number of iterations
    pub fn new(iterations: u32) -> Self {
        Self {
            started: Instant::now(),
            iterations,
            set_iterations: iterations,
            errors: 0,
            elapsed: None,
        }
    }

    /// Reset the benchmark timer
    pub fn reset(&mut self) {
        self.started = Instant::now();
        self.iterations = self.set_iterations;
        self.errors = 0;
    }

    /// Finish a simple benchmark
    pub fn finish0(&mut self) {
        self.elapsed = Some(self.started.elapsed());
    }

    /// Finish a simple benchmark, specifying number of iterations made
    pub fn finish(&mut self, iterations: Option<u32>, errors: Option<u32>) {
        self.elapsed = Some(self.started.elapsed());
        if let Some(i) = iterations {
            self.iterations = i;
        }
        if let Some(e) = errors {
            self.errors = e;
        }
    }

    /// Print a simple benchmark result
    pub fn print0(&self) {
        self.print(Some(self.iterations), Some(self.errors));
    }

    /// Print a simple benchmark result, specifying number of iterations made
    pub fn print(&self, iterations: Option<u32>, errors: Option<u32>) {
        println!("{}", self.to_string_for(iterations, errors));
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    /// Get a benchmark result
    pub fn result0(&self) -> BenchmarkResult {
        self.result(Some(self.iterations), Some(self.errors))
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    /// Get a benchmark result, specifying number of iterations made
    pub fn result(&self, iterations: Option<u32>, errors: Option<u32>) -> BenchmarkResult {
        let elapsed = self.elapsed.unwrap_or_else(|| self.started.elapsed());
        let it = iterations.unwrap_or(self.iterations);
        let errs = errors.unwrap_or(self.errors);
        BenchmarkResult {
            elapsed,
            iterations: it,
            errors: errs,
            speed: (f64::from(it - errs) / elapsed.as_secs_f64()) as u32,
        }
    }

    fn to_string_for(&self, iterations: Option<u32>, errors: Option<u32>) -> String {
        let result = self.result(iterations, errors);
        let elapsed = result.elapsed.as_secs_f64();
        format!(
            "{}\nIterations: {}, success: {}, errors: {}{}\n\
            Elapsed:\n {} secs ({} msecs)\n {} iters/s\n {} ns per iter",
            result_separator!(),
            format_number!(result.iterations).magenta(),
            format_number!(result.iterations - result.errors).green(),
            if result.errors > 0 {
                format_number!(result.errors).red()
            } else {
                "None".normal()
            },
            if result.errors > 0 {
                format!(
                    ", error rate: {}",
                    format!(
                        "{:.2} %",
                        (f64::from(result.errors) / f64::from(result.iterations) * 100.0)
                    )
                    .red()
                )
            } else {
                "".to_owned()
            },
            format!("{:.3}", elapsed).blue(),
            format!("{:.3}", elapsed * 1000.0).cyan(),
            format_number!(result.speed).yellow(),
            format_number!(1_000_000_000 / result.speed).magenta()
        )
    }

    /// Increment iterations inside benchmark
    ///
    /// Not required to use if the number of iterations is specified at benchmark creation or
    /// finish / print
    pub fn increment(&mut self) {
        self.iterations += 1;
    }

    /// Increment errors inside benchmark
    ///
    /// Not required to use if the number of errors is specified at benchmark finish / print
    pub fn increment_errors(&mut self) {
        self.errors += 1;
    }
}

fn ctable(titles: Option<Vec<&str>>, raw: bool) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .separators(
            &[prettytable::format::LinePosition::Title],
            prettytable::format::LineSeparator::new('-', '-', '-', '-'),
        )
        .padding(0, 1)
        .build();
    table.set_format(format);
    if let Some(tt) = titles {
        let mut titlevec: Vec<prettytable::Cell> = Vec::new();
        for t in tt {
            if raw {
                titlevec.push(prettytable::Cell::new(t));
            } else {
                titlevec.push(prettytable::Cell::new(t).style_spec("Fb"));
            }
        }
        table.set_titles(prettytable::Row::new(titlevec));
    };
    table
}

#[allow(clippy::cast_possible_truncation)]
fn separator(title: &str) -> colored::ColoredString {
    let size = terminal_size();
    let width = if let Some((Width(w), Height(_))) = size {
        w
    } else {
        40
    };
    (title.to_owned()
        + &(0..width - title.len() as u16)
            .map(|_| "-")
            .collect::<String>())
        .black()
}
