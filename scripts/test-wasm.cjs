// Exercise the actual Node and web WASM bindings against the committed oracle.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const root = path.resolve(__dirname, '..');
function verify(lib) {
  let count = 0;
  for (const line of fs.readFileSync(path.join(root,'tests/fixtures/parity.jsonl'),'utf8').trim().split('\n')) {
    const row = JSON.parse(line);let actual;
    switch(row.op) {
      case 'analyze': actual=lib.analyzeHapin(row.input,row.options);break;
      case 'convert': case 'analysis-convert': actual=lib.convert(row.input,row.options);break;
      case 'legacy': actual=lib[row.options.name](row.input,row.options.clean,row.options.easy);break;
      case 'array': actual=lib[row.options.name](row.input);break;
      case 'rules': actual=lib.segmentationRules();break;
      default: throw new Error(row.op);
    }
    assert.deepEqual(actual,row.expected,`WASM oracle ${count+1}: ${JSON.stringify(row.input)}`);count++;
  }
  assert.throws(()=>lib.analyzeHapin('ngh',{maxCandidates:0}));
  assert.throws(()=>lib.convert('ara',{from:'hapin',to:'future'}));
  const a=lib.analyzeHapin('a😀y',{});a.units[1].start=2;
  assert.throws(()=>lib.convert(a,{from:'hapin',to:'arabic'}));
  console.log(`WASM: ${count} structured oracle cases and invalid input checks passed`);
}
(async()=> {
  verify(require(path.join(root,'pkg/nodejs/hapin_utils_rs.js')));
  const web=await import(pathToFileURL(path.join(root,'pkg/web/hapin_utils_rs.js')));
  await web.default({module_or_path:fs.readFileSync(path.join(root,'pkg/web/hapin_utils_rs_bg.wasm'))});
  verify(web);
})().catch(error=>{console.error(error);process.exitCode=1;});
