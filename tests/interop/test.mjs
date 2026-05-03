import { execSync } from "child_process";
import { existsSync, mkdirSync, rmSync, readdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dir = dirname(fileURLToPath(import.meta.url));
const CACHE = join(__dir, ".tests-cache");
const GENERATED = join(__dir, "src", "generated");
const OUT_DIR = join(__dir, "output");

function run(cmd) {
  console.log("  >", cmd);
  execSync(cmd, { stdio: "inherit" });
}

function ensure(dir) {
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
}

console.log("\n=== Step 0: Install dependencies ===");
run(`cd ${__dir} && pnpm install`);

console.log("\n=== Step 1: Clone tests repo ===");
if (existsSync(CACHE)) rmSync(CACHE, { recursive: true });
run(`git clone --depth=1 https://github.com/specodec/tests ${CACHE}`);

console.log("\n=== Step 2: Generate vectors ===");
run(`cd ${CACHE} && pnpm install --frozen-lockfile`);
run(`cd ${CACHE} && node gen_types.mjs`);

const VEC_DIR = join(CACHE, "vectors");

console.log("\n=== Step 3: Generate emit code ===");
if (existsSync(GENERATED)) rmSync(GENERATED, { recursive: true });
ensure(GENERATED);

run(`cd ${__dir} && node_modules/.bin/tsp compile ${CACHE}/alltypes.tsp --emit=@specodec/typespec-emitter-rust \
  --option @specodec/typespec-emitter-rust.emitter-output-dir=${GENERATED}`);

const rustFiles = readdirSync(GENERATED).filter(f => f.endsWith(".rs"));
if (rustFiles.length > 0) {
  console.log("  ✓ Generated " + rustFiles.join(", "));
} else {
  console.error("  FAIL: No generated Rust files");
  process.exit(1);
}

console.log("\n=== Step 4: Generate test runner ===");
run(`cd ${__dir} && VEC_DIR=${VEC_DIR} node generate_emit_runner.mjs`);

console.log("\n=== Step 5: Run tests ===");
if (existsSync(OUT_DIR)) rmSync(OUT_DIR, { recursive: true });
ensure(OUT_DIR);

// Build and run with cargo
run(`cd ${__dir} && VEC_DIR=${VEC_DIR} OUT_DIR=${OUT_DIR} cargo run --release`);

console.log("\n=== ALL PASSED ===");