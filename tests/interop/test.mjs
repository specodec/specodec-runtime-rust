import { execSync } from 'child_process';
import { existsSync, mkdirSync, rmSync, readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dir = dirname(fileURLToPath(import.meta.url));
const LANG_ROOT = join(__dir, '..');
const TESTS_DIR = __dir;
const CACHE = join(TESTS_DIR, '.tests-cache');
const PROXY = process.env.PROXY || 'http://127.0.0.1:17890';

const BUILD_ARGS = `--build-arg HTTP_PROXY=${PROXY} --build-arg HTTPS_PROXY=${PROXY} --build-arg http_proxy=${PROXY} --build-arg https_proxy=${PROXY} --build-arg ALL_PROXY=${PROXY} --build-arg all_proxy=${PROXY}`;
const COMMON = `--network host ${BUILD_ARGS}`;

function run(cmd) {
  console.log('  >', cmd);
  execSync(cmd, { stdio: 'inherit' });
}

// Step 1: Clone tests repo
console.log('\n=== Step 1: Clone specodec/tests ===');
if (existsSync(CACHE)) {
  run(`git -C ${CACHE} pull`);
} else {
  run(`git clone --depth=1 https://github.com/specodec/tests ${CACHE}`);
}

// Step 2: Generate vectors + TS reference
console.log('\n=== Step 2: Generate vectors + output_ts ===');
run(`cd ${CACHE} && npm ci`);
run(`cd ${CACHE} && node gen_types.mjs`);
run(`cd ${CACHE} && node run_ts.mjs`);

const vectorsDir = join(CACHE, 'vectors');
const outputTsDir = join(CACHE, 'output_ts');

// Step 3: Interop test
console.log('\n=== Step 3: Interop test (podman build) ===');
const outputGo = join(TESTS_DIR, 'output_rust');
if (existsSync(outputGo)) rmSync(outputGo, { recursive: true });
mkdirSync(join(outputGo, 'scalars'), { recursive: true });

run(`cd ${TESTS_DIR} && podman build ${COMMON} -t specodec-interop-rust -f Containerfile \
  --build-context specodec-go=${LANG_ROOT} \
  --build-context run=${TESTS_DIR}/run \
  --build-context vectors=${vectorsDir} \
  --build-context output_ts=${outputTsDir} .`);

const container = execSync(`podman create specodec-interop-rust /bin/true`).toString().trim();
run(`podman cp ${container}:/app/output_rust/. ${outputGo}/`);
run(`podman rm ${container}`);

// Step 4: Emit compile test
console.log('\n=== Step 4: Emit compile test ===');
const emitGen = join(TESTS_DIR, '.emit-gen');
if (existsSync(emitGen)) rmSync(emitGen, { recursive: true });
mkdirSync(emitGen, { recursive: true });

run(`cd ${LANG_ROOT} && npx tsp compile ${CACHE}/alltypes.tsp --emit=@specodec/typespec-emitter-rust \
  --option @specodec/typespec-emitter-rust.emitter-output-dir=${emitGen}`);

run(`cd ${TESTS_DIR} && podman build ${COMMON} -t specodec-emit-rust -f Containerfile.emit \
  --build-context specodec-go=${LANG_ROOT} \
  --build-context emit=${TESTS_DIR}/emit \
  --build-context emit_gen=${emitGen} .`);

// Step 5: Emit roundtrip test
console.log('\n=== Step 5: Emit roundtrip test ===');
const outputEmitGo = join(TESTS_DIR, 'output_emit_rust');
if (existsSync(outputEmitGo)) rmSync(outputEmitGo, { recursive: true });
mkdirSync(outputEmitGo, { recursive: true });

run(`cd ${TESTS_DIR} && podman build ${COMMON} -t specodec-emit-run-rust -f Containerfile.emit-run \
  --build-context specodec-go=${LANG_ROOT} \
  --build-context emit=${TESTS_DIR}/emit \
  --build-context emit_gen=${emitGen} \
  --build-context vectors=${vectorsDir} .`);

const container2 = execSync(`podman create specodec-emit-run-rust /bin/true`).toString().trim();
run(`podman cp ${container2}:/app/output_emit_rust/. ${outputEmitGo}/`);
run(`podman rm ${container2}`);

// Step 6: Verify
console.log('\n=== Step 6: Verify ===');
run(`cd ${CACHE} && node verify.cjs --lang go --lang-output ${outputGo} --ts-output ${outputTsDir}`);
run(`cd ${CACHE} && node verify_emit.cjs --lang go --lang-output ${outputEmitGo} --ts-output ${join(CACHE, 'output_emit_ts')}`);

console.log('\n=== ALL PASSED ===');