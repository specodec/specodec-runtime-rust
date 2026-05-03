import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dir = path.dirname(fileURLToPath(import.meta.url));
const VEC_DIR = process.env.VEC_DIR || path.join(__dir, ".tests-cache", "vectors");

const manifestPath = path.join(VEC_DIR, "manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

const models = manifest.testModels;
const scalars = manifest.scalars;

function getReadMethod(type) {
  const map = {
    "int32": "read_int32",
    "int64": "read_int64",
    "uint32": "read_uint32",
    "uint64": "read_uint64",
    "float32": "read_float32",
    "float64": "read_float64",
    "string": "read_string",
    "bytes": "read_bytes",
    "bool": "read_bool",
  };
  return map[type] || "read_int32";
}

function getWriteMethod(type) {
  const map = {
    "int32": "write_int32",
    "int64": "write_int64",
    "uint32": "write_uint32",
    "uint64": "write_uint64",
    "float32": "write_float32",
    "float64": "write_float64",
    "string": "write_string",
    "bytes": "write_bytes",
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

let scalarFuncs = '';
let scalarCalls = '';
for (const [name, info] of Object.entries(scalars)) {
  const borrow = needsBorrow(info.type) ? "&" : "";
  const snake = toSnakeCase(name);
  const funcName = `test_scalar_${snake}`;
  scalarFuncs += `
fn ${funcName}(vec_dir: &str, out_dir: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("scalars/${name}.mp")) {
        let mut r = MsgPackReader::new(&b);
        if let Ok(val) = r.${getReadMethod(info.type)}() {
            let mut w = MsgPackWriter::new();
            w.${getWriteMethod(info.type)}(${borrow}val);
            if let Ok(_) = fs::write(Path::new(&out_dir).join("scalars/${name}.mp"), w.to_bytes()) {
                passed += 1;
            } else { println!("FAIL ${name} mp: write error"); failed += 1; }
        } else { println!("FAIL ${name} mp: read error"); failed += 1; }
    } else { println!("FAIL ${name} mp: file not found"); failed += 1; }
    (passed, failed)
}
`;
  scalarCalls += `    let (p, f) = ${funcName}(&vec_dir, &out_dir); passed += p; failed += f;\n`;
}

let modelFuncs = '';
let modelCalls = '';
for (const model of models) {
  const snake = toSnakeCase(model);
  const funcName = `test_model_${snake}`;
  modelFuncs += `
fn ${funcName}(vec_dir: &str, out_dir: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    // msgpack
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.msgpack")) {
        let mut r = MsgPackReader::new(&b);
        if let Ok(obj) = ${snake}_decode(&mut r) {
            let mut w = MsgPackWriter::new();
            ${snake}_write(&obj, &mut w);
            if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.msgpack"), w.to_bytes()) {
                passed += 1;
            } else { println!("FAIL ${model} mp: write error"); failed += 1; }
        } else { println!("FAIL ${model} mp: decode error"); failed += 1; }
    } else { println!("FAIL ${model} mp: file not found"); failed += 1; }
    // json
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.json")) {
        if let Ok(mut r) = JsonReader::new(&b) {
            if let Ok(obj) = ${snake}_decode(&mut r) {
                let mut w = JsonWriter::new();
                ${snake}_write(&obj, &mut w);
                if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.json"), w.to_bytes()) {
                    passed += 1;
                } else { println!("FAIL ${model} json: write error"); failed += 1; }
            } else { println!("FAIL ${model} json: decode error"); failed += 1; }
        } else { println!("FAIL ${model} json: reader error"); failed += 1; }
    } else { println!("FAIL ${model} json: file not found"); failed += 1; }
    // unformatted json
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.unformatted.json")) {
        if let Ok(mut r) = JsonReader::new(&b) {
            if let Ok(obj) = ${snake}_decode(&mut r) {
                let mut w = JsonWriter::new();
                ${snake}_write(&obj, &mut w);
                if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.unformatted.json"), w.to_bytes()) {
                    passed += 1;
                } else { println!("FAIL ${model} unformatted: write error"); failed += 1; }
            } else { println!("FAIL ${model} unformatted: decode error"); failed += 1; }
        } else { println!("FAIL ${model} unformatted: reader error"); failed += 1; }
    } else { println!("FAIL ${model} unformatted: file not found"); failed += 1; }
    // gron
    if let Ok(b) = fs::read(Path::new(&vec_dir).join("${model}.gron")) {
        let mut r = GronReader::new(&b);
        if let Ok(obj) = ${snake}_decode(&mut r) {
            let mut w = GronWriter::new();
            ${snake}_write(&obj, &mut w);
            if let Ok(_) = fs::write(Path::new(&out_dir).join("${model}.gron"), w.to_bytes()) {
                passed += 1;
            } else { println!("FAIL ${model} gron: write error"); failed += 1; }
        } else { println!("FAIL ${model} gron: decode error"); failed += 1; }
    } else { println!("FAIL ${model} gron: file not found"); failed += 1; }
    (passed, failed)
}
`;
  modelCalls += `    let (p, f) = ${funcName}(&vec_dir, &out_dir); passed += p; failed += f;\n`;
}

const code = `// Generated by generate_emit_runner.mjs. DO NOT EDIT.
mod generated;
use std::fs;
use std::path::Path;
use specodec::{MsgPackReader, MsgPackWriter, JsonReader, JsonWriter, GronReader, GronWriter, SpecReader, SpecWriter};
use generated::all_types_types::*;

${scalarFuncs}

${modelFuncs}

fn main() {
    let vec_dir = std::env::var("VEC_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fs::create_dir_all(&out_dir).unwrap();
    fs::create_dir_all(Path::new(&out_dir).join("scalars")).unwrap();

    let mut passed = 0u32;
    let mut failed = 0u32;

    // Scalar tests
${scalarCalls}

    // Object tests
${modelCalls}

    println!("emit-rust: {} passed, {} failed", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}
`;

const outFile = path.join(__dir, "src", "main.rs");
fs.writeFileSync(outFile, code);
console.log("Generated src/main.rs with " + models.length + " models + " + Object.keys(scalars).length + " scalars");