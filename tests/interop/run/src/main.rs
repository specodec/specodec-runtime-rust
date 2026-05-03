use std::fs;
use std::path::PathBuf;
use specodec::*;

fn vec_dir() -> PathBuf {
    PathBuf::from(std::env::var("VEC_DIR").unwrap_or_else(|_| "../vectors".into()))
}
fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap_or_else(|_| "../output_rust".into()))
}

fn main() {
    fs::create_dir_all(out_dir().join("scalars")).unwrap();

    let manifest_str = fs::read_to_string(vec_dir().join("manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    let schema_str = fs::read_to_string(vec_dir().join("typeschema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

    let mut scalar_results: Vec<(String, bool)> = Vec::new();
    let mut object_results: Vec<(String, bool, bool, bool)> = Vec::new();

    println!("Rust: processing scalars...");
    if let Some(scalars) = manifest["scalars"].as_object() {
        for (name, spec) in scalars {
            let stype = spec["type"].as_str().unwrap();
            let ref_buf = fs::read(vec_dir().join("scalars").join(format!("{name}.mp"))).unwrap();
            let mut r = MsgPackReader::new(&ref_buf);
            let mut w = MsgPackWriter::new();
            let res: Result<(), String> = match stype {
                "int32" => r.read_int32().map(|v| w.write_int32(v)).map_err(|e| e.message),
                "int64" => r.read_int64().map(|v| w.write_int64(v)).map_err(|e| e.message),
                "uint32" => r.read_uint32().map(|v| w.write_uint32(v)).map_err(|e| e.message),
                "uint64" => r.read_uint64().map(|v| w.write_uint64(v)).map_err(|e| e.message),
                "float32" => r.read_float32().map(|v| w.write_float32(v)).map_err(|e| e.message),
                "float64" => r.read_float64().map(|v| w.write_float64(v)).map_err(|e| e.message),
                "string" => r.read_string().map(|v| w.write_string(&v)).map_err(|e| e.message),
                "bytes" => r.read_bytes().map(|v| w.write_bytes(&v)).map_err(|e| e.message),
                "bool" => r.read_bool().map(|v| w.write_bool(v)).map_err(|e| e.message),
                t => Err(format!("unknown: {t}")),
            };
            match res {
                Ok(()) => { fs::write(out_dir().join("scalars").join(format!("{name}.mp")), w.to_bytes()).unwrap(); scalar_results.push((name.clone(), true)); }
                Err(e) => { println!("  FAIL {name}: {e}"); scalar_results.push((name.clone(), false)); }
            }
        }
    }

    println!("Rust: processing objects...");
    let test_models: Vec<String> = manifest["testModels"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    for name in &test_models {
        let (mp, json, gron) = process_object(name, &schema);
        object_results.push((name.clone(), mp, json, gron));
    }

    write_results(&scalar_results, &object_results);
    let fail = scalar_results.iter().filter(|(_, p)| !p).count() + object_results.iter().filter(|(_, mp, json, gron)| !mp || !json || !gron).count();
    let pass = scalar_results.len() + object_results.len() - fail;
    println!("Rust done: {pass} passed, {fail} failed");
    if fail > 0 { std::process::exit(1); }
}

// ═══════════════════════════════════
// Generic schema-driven decode/encode
// ═══════════════════════════════════

fn read_scalar<R: SpecReader>(r: &mut R, typ: &str) -> Result<serde_json::Value, SCodecError> {
    match typ {
        "string"  => Ok(serde_json::Value::String(r.read_string()?)),
        "boolean" => Ok(serde_json::Value::Bool(r.read_bool()?)),
        "int8" | "int16" | "int32" => Ok(serde_json::Value::Number(serde_json::Number::from(r.read_int32()?))),
        "int64"   => Ok(serde_json::json!({ "__i64": r.read_int64()? })),
        "uint8" | "uint16" | "uint32" => {
            let v = r.read_uint32()?;
            Ok(serde_json::Value::Number(serde_json::Number::from(v)))
        }
        "uint64"  => Ok(serde_json::json!({ "__u64": r.read_uint64()? })),
        "float32" => Ok(serde_json::Value::from(r.read_float32()? as f64)),
        "float64" => Ok(serde_json::Value::from(r.read_float64()?)),
        "bytes"   => {
            let b = r.read_bytes()?;
            Ok(serde_json::json!({ "__bytes": b }))
        }
        _ => Err(SCodecError::new(format!("unknown scalar: {typ}"))),
    }
}

fn decode_field<R: SpecReader>(r: &mut R, field: &serde_json::Value, schema: &serde_json::Value) -> Result<serde_json::Value, SCodecError> {
    let is_array = field.get("isArray").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_model = field.get("isModel").and_then(|v| v.as_bool()).unwrap_or(false);
    let typ = field["type"].as_str().unwrap();

    if is_array {
        let mut arr = Vec::new();
        r.begin_array()?;
        while r.has_next_element()? {
            if is_model {
                arr.push(decode_model(r, typ, schema)?);
            } else {
                arr.push(read_scalar(r, typ)?);
            }
        }
        r.end_array()?;
        Ok(serde_json::Value::Array(arr))
    } else if is_model {
        decode_model(r, typ, schema)
    } else {
        read_scalar(r, typ)
    }
}

fn decode_model<R: SpecReader>(r: &mut R, model_name: &str, schema: &serde_json::Value) -> Result<serde_json::Value, SCodecError> {
    let model_schema = &schema[model_name];
    let fields = model_schema["fields"].as_array().unwrap();
    let mut map = serde_json::Map::new();
    r.begin_object()?;
    while r.has_next_field()? {
        let k = r.read_field_name()?;
        let field = fields.iter().find(|f| f["name"].as_str() == Some(&k));
        if let Some(f) = field {
            map.insert(k, decode_field(r, f, schema)?);
        } else {
            r.skip()?;
        }
    }
    r.end_object()?;
    Ok(serde_json::Value::Object(map))
}

fn read_scalar_gron(r: &mut GronReader, typ: &str) -> Result<serde_json::Value, SCodecError> {
    match typ {
        "string"  => Ok(serde_json::Value::String(r.read_string()?)),
        "boolean" => Ok(serde_json::Value::Bool(r.read_bool()?)),
        "int8" | "int16" | "int32" => Ok(serde_json::Value::Number(serde_json::Number::from(r.read_int32()?))),
        "int64"   => Ok(serde_json::json!({ "__i64": r.read_int64()? })),
        "uint8" | "uint16" | "uint32" => {
            let v = r.read_uint32()?;
            Ok(serde_json::Value::Number(serde_json::Number::from(v)))
        }
        "uint64"  => Ok(serde_json::json!({ "__u64": r.read_uint64()? })),
        "float32" => Ok(serde_json::Value::from(r.read_float32()? as f64)),
        "float64" => Ok(serde_json::Value::from(r.read_float64()?)),
        "bytes"   => {
            let b = r.read_bytes()?;
            Ok(serde_json::json!({ "__bytes": b }))
        }
        _ => Err(SCodecError::new(format!("unknown scalar: {typ}"))),
    }
}

fn decode_field_gron(r: &mut GronReader, field: &serde_json::Value, schema: &serde_json::Value) -> Result<serde_json::Value, SCodecError> {
    let is_array = field.get("isArray").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_model = field.get("isModel").and_then(|v| v.as_bool()).unwrap_or(false);
    let typ = field["type"].as_str().unwrap();

    if is_array {
        let mut arr = Vec::new();
        r.begin_array()?;
        while r.has_next_element()? {
            r.next_element()?;
            if is_model {
                arr.push(decode_model_gron(r, typ, schema)?);
            } else {
                arr.push(read_scalar_gron(r, typ)?);
            }
        }
        r.end_array()?;
        Ok(serde_json::Value::Array(arr))
    } else if is_model {
        decode_model_gron(r, typ, schema)
    } else {
        read_scalar_gron(r, typ)
    }
}

fn decode_model_gron(r: &mut GronReader, model_name: &str, schema: &serde_json::Value) -> Result<serde_json::Value, SCodecError> {
    let model_schema = &schema[model_name];
    let fields = model_schema["fields"].as_array().unwrap();
    let mut map = serde_json::Map::new();
    r.begin_object()?;
    while r.has_next_field()? {
        let k = r.read_field_name()?;
        let field = fields.iter().find(|f| f["name"].as_str() == Some(&k));
        if let Some(f) = field {
            map.insert(k, decode_field_gron(r, f, schema)?);
        } else {
            r.skip()?;
        }
    }
    r.end_object()?;
    Ok(serde_json::Value::Object(map))
}

fn write_scalar_mp(w: &mut MsgPackWriter, val: &serde_json::Value, typ: &str) {
    match typ {
        "string"  => w.write_string(val.as_str().unwrap()),
        "boolean" => w.write_bool(val.as_bool().unwrap()),
        "int8" | "int16" | "int32" => w.write_int32(val.as_i64().unwrap() as i32),
        "int64"   => w.write_int64(val["__i64"].as_i64().unwrap()),
        "uint8" | "uint16" | "uint32" => w.write_uint32(val.as_u64().unwrap() as u32),
        "uint64"  => w.write_uint64(val["__u64"].as_u64().unwrap()),
        "float32" => w.write_float32(val.as_f64().unwrap() as f32),
        "float64" => w.write_float64(val.as_f64().unwrap()),
        "bytes"   => {
            let arr = val["__bytes"].as_array().unwrap();
            let b: Vec<u8> = arr.iter().map(|v| v.as_u64().unwrap() as u8).collect();
            w.write_bytes(&b);
        }
        _ => panic!("unknown scalar: {typ}"),
    }
}

fn encode_field_mp(w: &mut MsgPackWriter, val: &serde_json::Value, field: &serde_json::Value, schema: &serde_json::Value) {
    let is_array = field.get("isArray").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_model = field.get("isModel").and_then(|v| v.as_bool()).unwrap_or(false);
    let typ = field["type"].as_str().unwrap();

    if is_array {
        let arr = val.as_array().unwrap();
        w.begin_array(arr.len());
        for item in arr {
            if is_model { encode_model_inline_mp(w, item, typ, schema); }
            else { write_scalar_mp(w, item, typ); }
        }
        w.end_array();
    } else if is_model {
        encode_model_inline_mp(w, val, typ, schema);
    } else {
        write_scalar_mp(w, val, typ);
    }
}

fn encode_model_mp(val: &serde_json::Value, model_name: &str, schema: &serde_json::Value) -> Vec<u8> {
    let mut w = MsgPackWriter::new();
    encode_model_inline_mp(&mut w, val, model_name, schema);
    w.to_bytes()
}

fn encode_model_inline_mp(w: &mut MsgPackWriter, val: &serde_json::Value, model_name: &str, schema: &serde_json::Value) {
    let model_schema = &schema[model_name];
    let fields = model_schema["fields"].as_array().unwrap();
    let obj = val.as_object().unwrap();
    let count = fields.iter().filter(|f| {
        let name = f["name"].as_str().unwrap();
        let opt = f.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        !opt || obj.contains_key(name)
    }).count();
    w.begin_object(count);
    for f in fields {
        let name = f["name"].as_str().unwrap();
        let opt = f.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        if opt && !obj.contains_key(name) { continue; }
        w.write_field(name);
        encode_field_mp(w, &obj[name], f, schema);
    }
    w.end_object();
}

fn write_scalar_json(w: &mut JsonWriter, val: &serde_json::Value, typ: &str) {
    match typ {
        "string"  => w.write_string(val.as_str().unwrap()),
        "boolean" => w.write_bool(val.as_bool().unwrap()),
        "int8" | "int16" | "int32" => w.write_int32(val.as_i64().unwrap() as i32),
        "int64"   => w.write_int64(val["__i64"].as_i64().unwrap()),
        "uint8" | "uint16" | "uint32" => w.write_uint32(val.as_u64().unwrap() as u32),
        "uint64"  => w.write_uint64(val["__u64"].as_u64().unwrap()),
        "float32" => w.write_float32(val.as_f64().unwrap() as f32),
        "float64" => w.write_float64(val.as_f64().unwrap()),
        "bytes"   => {
            let arr = val["__bytes"].as_array().unwrap();
            let b: Vec<u8> = arr.iter().map(|v| v.as_u64().unwrap() as u8).collect();
            w.write_bytes(&b);
        }
        _ => panic!("unknown scalar: {typ}"),
    }
}

fn encode_field_json(w: &mut JsonWriter, val: &serde_json::Value, field: &serde_json::Value, schema: &serde_json::Value) {
    let is_array = field.get("isArray").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_model = field.get("isModel").and_then(|v| v.as_bool()).unwrap_or(false);
    let typ = field["type"].as_str().unwrap();

    if is_array {
        let arr = val.as_array().unwrap();
        w.begin_array(arr.len());
        for item in arr {
            w.next_element();
            if is_model { encode_model_inline_json(w, item, typ, schema); }
            else { write_scalar_json(w, item, typ); }
        }
        w.end_array();
    } else if is_model {
        encode_model_inline_json(w, val, typ, schema);
    } else {
        write_scalar_json(w, val, typ);
    }
}

fn encode_model_json(val: &serde_json::Value, model_name: &str, schema: &serde_json::Value) -> Vec<u8> {
    let mut w = JsonWriter::new();
    encode_model_inline_json(&mut w, val, model_name, schema);
    w.to_bytes()
}

fn encode_model_inline_json(w: &mut JsonWriter, val: &serde_json::Value, model_name: &str, schema: &serde_json::Value) {
    let model_schema = &schema[model_name];
    let fields = model_schema["fields"].as_array().unwrap();
    let obj = val.as_object().unwrap();
    let count = fields.iter().filter(|f| {
        let name = f["name"].as_str().unwrap();
        let opt = f.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        !opt || obj.contains_key(name)
    }).count();
    w.begin_object(count);
    for f in fields {
        let name = f["name"].as_str().unwrap();
        let opt = f.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        if opt && !obj.contains_key(name) { continue; }
        w.write_field(name);
        encode_field_json(w, &obj[name], f, schema);
    }
    w.end_object();
}

fn write_scalar_gron(w: &mut GronWriter, val: &serde_json::Value, typ: &str) {
    match typ {
        "string"  => w.write_string(val.as_str().unwrap()),
        "boolean" => w.write_bool(val.as_bool().unwrap()),
        "int8" | "int16" | "int32" => w.write_int32(val.as_i64().unwrap() as i32),
        "int64"   => w.write_int64(val["__i64"].as_i64().unwrap()),
        "uint8" | "uint16" | "uint32" => w.write_uint32(val.as_u64().unwrap() as u32),
        "uint64"  => w.write_uint64(val["__u64"].as_u64().unwrap()),
        "float32" => w.write_float32(val.as_f64().unwrap() as f32),
        "float64" => w.write_float64(val.as_f64().unwrap()),
        "bytes"   => {
            let arr = val["__bytes"].as_array().unwrap();
            let b: Vec<u8> = arr.iter().map(|v| v.as_u64().unwrap() as u8).collect();
            w.write_bytes(&b);
        }
        _ => panic!("unknown scalar: {typ}"),
    }
}

fn encode_field_gron(w: &mut GronWriter, val: &serde_json::Value, field: &serde_json::Value, schema: &serde_json::Value) {
    let is_array = field.get("isArray").and_then(|v| v.as_bool()).unwrap_or(false);
    let is_model = field.get("isModel").and_then(|v| v.as_bool()).unwrap_or(false);
    let typ = field["type"].as_str().unwrap();

    if is_array {
        let arr = val.as_array().unwrap();
        w.begin_array(arr.len());
        for item in arr {
            w.next_element();
            if is_model { encode_model_inline_gron(w, item, typ, schema); }
            else { write_scalar_gron(w, item, typ); }
        }
        w.end_array();
    } else if is_model {
        encode_model_inline_gron(w, val, typ, schema);
    } else {
        write_scalar_gron(w, val, typ);
    }
}

fn encode_model_gron(val: &serde_json::Value, model_name: &str, schema: &serde_json::Value) -> Vec<u8> {
    let mut w = GronWriter::new();
    encode_model_inline_gron(&mut w, val, model_name, schema);
    w.to_bytes()
}

fn encode_model_inline_gron(w: &mut GronWriter, val: &serde_json::Value, model_name: &str, schema: &serde_json::Value) {
    let model_schema = &schema[model_name];
    let fields = model_schema["fields"].as_array().unwrap();
    let obj = val.as_object().unwrap();
    let count = fields.iter().filter(|f| {
        let name = f["name"].as_str().unwrap();
        let opt = f.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        !opt || obj.contains_key(name)
    }).count();
    w.begin_object(count);
    for f in fields {
        let name = f["name"].as_str().unwrap();
        let opt = f.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        if opt && !obj.contains_key(name) { continue; }
        w.write_field(name);
        encode_field_gron(w, &obj[name], f, schema);
    }
    w.end_object();
}

fn process_object(name: &str, schema: &serde_json::Value) -> (bool, bool, bool) {
    let mut mp_ok = false;
    let mut json_ok = false;
    let mut gron_ok = false;

    match (|| -> Result<(), String> {
        let mp_buf = fs::read(vec_dir().join(format!("{name}.msgpack"))).map_err(|e| format!("mp: {e}"))?;
        let mut r = MsgPackReader::new(&mp_buf);
        let decoded = decode_model(&mut r, name, schema).map_err(|e| format!("mp decode: {}", e.message))?;
        let encoded = encode_model_mp(&decoded, name, schema);
        fs::write(out_dir().join(format!("{name}.msgpack")), &encoded).map_err(|e| format!("mp write: {e}"))?;
        Ok(())
    })() {
        Ok(()) => mp_ok = true,
        Err(e) => println!("  FAIL {name}.msgpack: {e}"),
    }

    let mut compact_encoded: Option<Vec<u8>> = None;
    match (|| -> Result<(), String> {
        let json_buf = fs::read(vec_dir().join(format!("{name}.json"))).map_err(|e| format!("json: {e}"))?;
        let mut r = JsonReader::new(&json_buf).map_err(|e| format!("json init: {}", e.message))?;
        let decoded = decode_model(&mut r, name, schema).map_err(|e| format!("json decode: {}", e.message))?;
        let enc = encode_model_json(&decoded, name, schema);
        fs::write(out_dir().join(format!("{name}.json")), &enc).map_err(|e| format!("json write: {e}"))?;
        compact_encoded = Some(enc);
        Ok(())
    })() {
        Ok(()) => json_ok = true,
        Err(e) => println!("  FAIL {name}.json: {e}"),
    }

    if let Some(ref compact) = compact_encoded {
        let pretty_path = vec_dir().join(format!("{name}.pretty.json"));
        if pretty_path.exists() {
            match (|| -> Result<(), String> {
                let pretty_buf = fs::read(&pretty_path).map_err(|e| format!("pretty json read: {e}"))?;
                let mut r2 = JsonReader::new(&pretty_buf).map_err(|e| format!("pretty json init: {}", e.message))?;
                let decoded2 = decode_model(&mut r2, name, schema).map_err(|e| format!("pretty json decode: {}", e.message))?;
                let pretty_encoded = encode_model_json(&decoded2, name, schema);
                if pretty_encoded != *compact {
                    return Err("pretty.json: re-encoded bytes differ".into());
                }
                Ok(())
            })() {
                Err(e) => { println!("  FAIL {name}.pretty.json: {e}"); json_ok = false; }
                _ => {}
            }
        }
    }

    match (|| -> Result<(), String> {
        let gron_buf = fs::read(vec_dir().join(format!("{name}.gron"))).map_err(|e| format!("gron: {e}"))?;
        let mut r = GronReader::new(&gron_buf);
        let decoded = decode_model_gron(&mut r, name, schema).map_err(|e| format!("gron decode: {}", e.message))?;
        let encoded = encode_model_gron(&decoded, name, schema);
        fs::write(out_dir().join(format!("{name}.gron")), &encoded).map_err(|e| format!("gron write: {e}"))?;
        Ok(())
    })() {
        Ok(()) => gron_ok = true,
        Err(e) => println!("  FAIL {name}.gron: {e}"),
    }

    (mp_ok, json_ok, gron_ok)
}

fn write_results(scalars: &[(String, bool)], objects: &[(String, bool, bool, bool)]) {
    let mut s = String::from("{\"scalars\":{");
    for (i, (name, pass)) in scalars.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("\"{name}\":{{\"pass\":{pass}}}"));
    }
    s.push_str("},\"objects\":{");
    for (i, (name, mp, json, gron)) in objects.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("\"{name}\":{{\"mp\":{mp},\"json\":{json},\"gron\":{gron}}}"));
    }
    s.push_str("}}");
    fs::write(out_dir().join("results.json"), s).unwrap();
}
