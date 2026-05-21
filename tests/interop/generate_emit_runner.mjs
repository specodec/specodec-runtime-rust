import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dir = path.dirname(fileURLToPath(import.meta.url));
const VEC_DIR = process.env.VEC_DIR || path.join(__dir, "vectors");

const manifestPath = path.join(VEC_DIR, "manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

const models = [...(manifest.testModels || []), ...(manifest.testUnions || [])];
const scalars = manifest.scalars;
const modelNamespaces = manifest.modelNamespaces || {};
const testUnions = new Set(manifest.testUnions || []);
function isUnionTest(name) { return testUnions.has(name); }
function unionNameOf(testName) { return testName.replace(/_[^_]+$/, ''); }

const generatedDir = path.join(__dir, "src", "generated");

function readMethod(type) {
  const map = {
    "int32": "read_int32", "int64": "read_int64",
    "uint32": "read_uint32", "uint64": "read_uint64",
    "float32": "read_float32", "float64": "read_float64",
    "string": "read_string", "bytes": "read_bytes",
    "bool": "read_bool",
  };
  return map[type] || "read_int32";
}

function writeMethod(type) {
  const map = {
    "int32": "write_int32", "int64": "write_int64",
    "uint32": "write_uint32", "uint64": "write_uint64",
    "float32": "write_float32", "float64": "write_float64",
    "string": "write_string", "bytes": "write_bytes",
    "bool": "write_bool",
  };
  return map[type] || "write_int32";
}

function needsBorrow(type) {
  return type === "string" || type === "bytes";
}

function toSnakeCase(name) {
  let snake = name.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
  snake = snake.replace(/\./g, '_').replace(/-/g, '_');
  return snake;
}

function nsToSnake(ns) {
  return ns.split('.').map(p => toSnakeCase(p)).join('_');
}

// ── Scan generated .rs files to map model→module ──
const modelModule = {};
if (fs.existsSync(generatedDir)) {
  const rsFiles = fs.readdirSync(generatedDir)
    .filter(f => f.endsWith(".rs") && f !== "mod.rs");
  for (const f of rsFiles) {
    const modName = f.replace(".rs", "");
    const content = fs.readFileSync(path.join(generatedDir, f), "utf-8");
    for (const model of models) {
      const snake = toSnakeCase(model);
      if (content.includes("fn " + snake + "_decode")) {
        modelModule[model] = modName;
      }
      if (isUnionTest(model)) {
        const unionSnake = toSnakeCase(unionNameOf(model));
        if (content.includes("fn " + unionSnake + "_decode")) {
          modelModule[model] = modName;
        }
      }
    }
  }
}
// Default for any model not found
for (const model of models) {
  if (!modelModule[model]) modelModule[model] = "all_types_types";
}

// ── Group models: by namespace if available, else by generated module ──
let testGroups = {};
const hasModelNs = Object.keys(modelNamespaces).length > 0;

if (hasModelNs) {
  for (const model of models) {
    const ns = modelNamespaces[model] || "AllTypes";
    if (!testGroups[ns]) testGroups[ns] = [];
    testGroups[ns].push(model);
  }
} else {
  // Group by which generated module they live in
  for (const model of models) {
    const mod = modelModule[model];
    let key = mod;
    if (!testGroups[key]) testGroups[key] = [];
    testGroups[key].push(model);
  }
}

// ── Test module name from group key ──
function groupKeyToModName(key) {
  if (hasModelNs) {
    // Namespace path → module name
    const snake = nsToSnake(key);
    return "test_" + snake;
  }
  // Generated module name → simplified test name
  // "all_types_types" → "test_all_types", "all_types_nested_types" → "test_all_types_nested"
  const simple = key.replace(/_types$/, '');
  return "test_" + simple;
}

// ── Scalar test function generator ──
function generateScalarFunc(name, info) {
  const borrow = needsBorrow(info.type) ? "&" : "";
  const snake = toSnakeCase(name);
  const funcName = `test_scalar_${snake}`;
  return `
fn ${funcName}(vec_dir: &str, out_dir: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("scalars/${name}.mp")) {
        let mut r = MsgPackReader::new(&b);
        if let Ok(val) = r.${readMethod(info.type)}() {
            let mut w = MsgPackWriter::new();
            w.${writeMethod(info.type)}(${borrow}val);
            if let Ok(_) = fs::write(Path::new(&out_dir).join("scalars/${name}.mp"), w.to_bytes()) {
                passed += 1;
            } else { println!("FAIL ${name} mp: write error"); failed += 1; }
        } else { println!("FAIL ${name} mp: read error"); failed += 1; }
    } else { println!("FAIL ${name} mp: file not found"); failed += 1; }
    (passed, failed)
}
`;
}

// ── Model test function generator ──
function generateModelFunc(model) {
  const snake = toSnakeCase(model);
  const funcName = `test_model_${snake}`;
  const codecSnake = isUnionTest(model) ? toSnakeCase(unionNameOf(model)) : snake;
  return `
fn ${funcName}(vec_dir: &str, out_dir: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    // msgpack
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.msgpack")) {
        let mut r = MsgPackReader::new(&b);
        if let Ok(obj) = ${codecSnake}_decode(&mut r) {
            let mut w = MsgPackWriter::new();
            ${codecSnake}_write(&obj, &mut w);
            if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.msgpack"), w.to_bytes()) {
                passed += 1;
            } else { println!("FAIL ${model} mp: write error"); failed += 1; }
        } else { println!("FAIL ${model} mp: decode error"); failed += 1; }
    } else { println!("FAIL ${model} mp: file not found"); failed += 1; }
    // json
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.json")) {
        if let Ok(mut r) = JsonReader::new(&b) {
            if let Ok(obj) = ${codecSnake}_decode(&mut r) {
                let mut w = JsonWriter::new();
                ${codecSnake}_write(&obj, &mut w);
                if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.json"), w.to_bytes()) {
                    passed += 1;
                } else { println!("FAIL ${model} json: write error"); failed += 1; }
            } else { println!("FAIL ${model} json: decode error"); failed += 1; }
        } else { println!("FAIL ${model} json: reader error"); failed += 1; }
    } else { println!("FAIL ${model} json: file not found"); failed += 1; }
    // unformatted json
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.unformatted.json")) {
        if let Ok(mut r) = JsonReader::new(&b) {
            if let Ok(obj) = ${codecSnake}_decode(&mut r) {
                let mut w = JsonWriter::new();
                ${codecSnake}_write(&obj, &mut w);
                if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.unformatted.json"), w.to_bytes()) {
                    passed += 1;
                } else { println!("FAIL ${model} unformatted: write error"); failed += 1; }
            } else { println!("FAIL ${model} unformatted: decode error"); failed += 1; }
        } else { println!("FAIL ${model} unformatted: reader error"); failed += 1; }
    } else { println!("FAIL ${model} unformatted: file not found"); failed += 1; }
    // gron
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.gron")) {
        let mut r = GronReader::new(&b);
        if let Ok(obj) = ${codecSnake}_decode(&mut r) {
            let mut w = GronWriter::new();
            ${codecSnake}_write(&obj, &mut w);
            if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.gron"), w.to_bytes()) {
                passed += 1;
            } else { println!("FAIL ${model} gron: write error"); failed += 1; }
        } else { println!("FAIL ${model} gron: decode error"); failed += 1; }
    } else { println!("FAIL ${model} gron: file not found"); failed += 1; }
    (passed, failed)
}
`;
}

// ── Determine which generated modules a group of models needs ──
function getModulesForGroup(groupModels) {
  const mods = new Set();
  for (const model of groupModels) {
    mods.add(modelModule[model]);
  }
  return [...mods];
}

// ── Generate test files ──
const srcDir = path.join(__dir, "src");
const modDecls = [];
const modCalls = [];

// 1. Scalars file
if (Object.keys(scalars).length > 0) {
  let scalarFuncs = '';
  let scalarCalls = '';
  for (const [name, info] of Object.entries(scalars)) {
    scalarFuncs += generateScalarFunc(name, info);
    const sname = toSnakeCase(name);
    scalarCalls += `    let (p, f) = test_scalar_${sname}(vec_dir, out_dir); passed += p; failed += f;\n`;
  }
  const scalarCode = `// Generated by generate_emit_runner.mjs. DO NOT EDIT.
use std::fs;
use std::path::Path;
use specodec::{MsgPackReader, MsgPackWriter, SpecReader, SpecWriter};

${scalarFuncs}

pub fn run(vec_dir: &str, out_dir: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
${scalarCalls}
    (passed, failed)
}
`;
  fs.writeFileSync(path.join(srcDir, "test_scalars.rs"), scalarCode);
  console.log("Generated src/test_scalars.rs (" + Object.keys(scalars).length + " scalars)");
  modDecls.push("mod test_scalars;");
  modCalls.push("    let (p, f) = test_scalars::run(&vec_dir, &out_dir); passed += p; failed += f;");
}

// 2. Model test files per group
const usedModules = new Set();
for (const [key, groupModels] of Object.entries(testGroups)) {
  const testModName = groupKeyToModName(key);
  const modules = getModulesForGroup(groupModels);
  const useLines = modules.map(m => `use crate::generated::${m}::*;`).join("\n");

  let modelFuncs = '';
  let modelCalls = '';
  for (const model of groupModels) {
    const snake = toSnakeCase(model);
    modelFuncs += generateModelFunc(model);
    modelCalls += `    let (p, f) = test_model_${snake}(vec_dir, out_dir); passed += p; failed += f;\n`;
  }

  const testCode = `// Generated by generate_emit_runner.mjs. DO NOT EDIT.
use std::fs;
use std::path::Path;
use specodec::{MsgPackReader, MsgPackWriter, JsonReader, JsonWriter, GronReader, GronWriter, SpecReader, SpecWriter};
${useLines}

${modelFuncs}

pub fn run(vec_dir: &str, out_dir: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
${modelCalls}
    (passed, failed)
}
`;
  const fileName = testModName + ".rs";
  fs.writeFileSync(path.join(srcDir, fileName), testCode);
  console.log("Generated src/" + fileName + " (" + groupModels.length + " models)");
  modDecls.push("mod " + testModName + ";");
  modCalls.push("    let (p, f) = " + testModName + "::run(&vec_dir, &out_dir); passed += p; failed += f;");
}

// ── Generate main.rs ──
const mainCode = `// Generated by generate_emit_runner.mjs. DO NOT EDIT.
pub mod generated;
use std::fs;
use std::path::Path;

${modDecls.join("\n")}

fn main() {
    let vec_dir = std::env::var("VEC_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fs::create_dir_all(&out_dir).unwrap();
    fs::create_dir_all(Path::new(&out_dir).join("scalars")).unwrap();

    let mut passed = 0u32;
    let mut failed = 0u32;

${modCalls.join("\n")}

    println!("emit-rust: {} passed, {} failed", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}
`;

fs.writeFileSync(path.join(srcDir, "main.rs"), mainCode);
console.log("Generated src/main.rs with " + modDecls.length + " test modules");
