// Run: cd tests/ryu && rustc test_ryu.rs -o test_ryu && ./test_ryu
use std::fs;
use std::path::Path;

// Copy of ryu functions inline for test compilation
mod ryu {
    pub mod ryu_f32 {
        mod ryu_math { include!("../../src/ryu/ryu_math.rs"); }
        mod tables_f32 { include!("../../src/ryu/tables_f32.rs"); }
        include!("../../src/ryu/ryu_f32.rs");
    }
    pub mod ryu_f64 {
        mod ryu_math { include!("../../src/ryu/ryu_math.rs"); }
        mod tables_f64 { include!("../../src/ryu/tables_f64.rs"); }
        include!("../../src/ryu/ryu_f64.rs");
    }
}

fn load_tests(filename: &str) -> Vec<f64> {
    let content = fs::read_to_string(Path::new(filename)).unwrap();
    content.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.parse().unwrap())
        .collect()
}

fn load_expected(filename: &str) -> Vec<String> {
    let content = fs::read_to_string(Path::new(filename)).unwrap();
    content.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn load_coverage(filename: &str) -> Vec<f64> {
    let content = fs::read_to_string(Path::new(filename)).unwrap();
    content.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && l.chars().next().map_or(false, |c| c.is_ascii_digit()))
        .map(|l| {
            let v = if let Some(idx) = l.find('#') { &l[..idx] } else { l };
            v.trim().parse().unwrap()
        })
        .collect()
}

fn main() {
    let mut passed = 0i32;
    let mut failed = 0i32;

    println!("=== Float32 Original (125 tests) ===");
    let f32in = load_tests("test_cases_f32.txt");
    let f32exp = load_expected("expected_f32.txt");
    for i in 0..f32in.len().min(f32exp.len()) {
        let result = ryu::ryu_f32::float32_to_string(f32in[i] as f32);
        if result == f32exp[i] {
            passed += 1;
        } else {
            failed += 1;
            if failed <= 5 { println!("FAIL: {} => {} (expected {})", f32in[i], result, f32exp[i]); }
        }
    }
    println!("{}/{}\n", f32in.len(), f32in.len());

    println!("=== Float64 Original (102 tests) ===");
    let f64in = load_tests("test_cases_f64.txt");
    let f64exp = load_expected("expected_f64.txt");
    for i in 0..f64in.len().min(f64exp.len()) {
        let result = ryu::ryu_f64::float64_to_string(f64in[i]);
        if result == f64exp[i] {
            passed += 1;
        } else {
            failed += 1;
            if failed <= 5 { println!("FAIL: {} => {} (expected {})", f64in[i], result, f64exp[i]); }
        }
    }
    println!("{}/{}\n", f64in.len(), f64in.len());

    println!("=== Float32 Coverage (78 tests) ===");
    let c32in = load_coverage("test_cases_table_coverage.txt");
    let c32exp = load_expected("expected_table_coverage.txt");
    let n = c32in.len().min(c32exp.len());
    for i in 0..n {
        let result = ryu::ryu_f32::float32_to_string(c32in[i] as f32);
        if result == c32exp[i] {
            passed += 1;
        } else {
            failed += 1;
            if failed <= 5 { println!("FAIL: {} => {} (expected {})", c32in[i], result, c32exp[i]); }
        }
    }
    println!("{}/{}\n", n, n);

    println!("=== Float64 Coverage (616 tests) ===");
    let c64in = load_coverage("test_cases_f64_table_coverage.txt");
    let c64exp = load_expected("expected_f64_table_coverage.txt");
    let n = c64in.len().min(c64exp.len());
    for i in 0..n {
        let result = ryu::ryu_f64::float64_to_string(c64in[i]);
        if result == c64exp[i] {
            passed += 1;
        } else {
            failed += 1;
            if failed <= 5 { println!("FAIL: {} => {} (expected {})", c64in[i], result, c64exp[i]); }
        }
    }
    println!("{}/{}\n", n, n);

    println!("=== TOTAL: {}/{} ===", passed, passed + failed);
    if failed > 0 { std::process::exit(1); }
}
