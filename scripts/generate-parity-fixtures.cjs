// Maintainer-only oracle generator; Rust builds/tests never require Node or JS.
// Usage: node scripts/generate-parity-fixtures.cjs /absolute/path/to/hapin-utils
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const root = path.resolve(__dirname, '..');
const source = path.resolve(process.argv[2] || '../hapin-utils');
const pkg = require(path.join(source, 'package.json'));
if (pkg.version !== '5.0.0') throw new Error('Oracle must be hapin-utils 5.0.0');
const lib = require(path.join(source, 'dist/index.js'));
const sourceCommit = cp.execFileSync('git', ['-C', source, 'rev-parse', 'HEAD'], {encoding:'utf8'}).trim();
const dirty = cp.execFileSync('git', ['-C', source, 'status', '--porcelain'], {encoding:'utf8'}).trim();
if (dirty) throw new Error('Oracle checkout must be clean');
const rows = [];
const add = (op, input, options, expected) => rows.push({op, input, options, expected});
const focused = [
 '', 'ara', 'arzangha', 'shanghai', 'shanghaida', 'shanghaigha', 'shanghaixyz', 'tunghwa', 'junghwa', 'hungha', 'hunghwa', 'shinghaw', 'wanghung',
 'amangul', 'ngul', 'ngye', 'ngxe', 'angye', 'ashana', 'ashanada', 'ashanadan', 'ashanase', 'ashanagha', 'ashanane', 'ashananeng', 'eshanada', 'eshanagha', 'yeshat', 'jarashan', 'ashanaxyz', 'ashat',
 'aldenge', 'angmye', 'angemye', 'ang', 'berdyengye', 'jyekyengye', 'jyengye', 'jyengyem', 'songe', 'syekyengye', 'mange', 'yerkengye',
 'shsh', 'hh', 'tsh', 'tshsh', 'nghngh', 'NGH', 'SHANGHAI', 'AShANA', 'ngh ngh ngh', 'gi', 'k xa', 'gxa', 'ghxa', 'a\nxa',
 'xa xo xe xu yu yi yo yw ya c', 'a x', 'xx xy yy y', 'İK', 'A  xa\t"Hapin  w"\n😀y', '"abc', '"', '"a\\"b"', '"\\😀"',
 'a\\b', 'n\u200bgh', 'g\u200bh', 'a, b; c? d- e! f # $ % & / : @', "n'gh", '  a  ', '\t\r\n', '\u00a0\u1680\u2007\u2028\u202f\ufeff', '\u0085\u0301\u200c\u200d', 'a😀y',
];
const directions = [ ['hapin','arabic'], ['hapin','ipa-cna'], ['arabic','hapin'], ['arabic','ipa-cna'], ['cyrillic','hapin'], ['cyrillic','arabic'], ['cyrillic','ipa-cna'] ];
for (const input of focused) {
 for (const options of [{}, {rules:'legacy'}, {inputFormat:'legacy-boundaries'}, {maxCandidates:1}, {maxCandidates:2}]) add('analyze',input,options,lib.analyzeHapin(input,options));
 for (const [from,to] of directions.slice(0,2)) {
  for (const extra of [{}, {rules:'legacy'}, {errors:'strict'}, {formatting:'legacy'}, {inputFormat:'legacy-boundaries'}]) {
   const options = {from,to,...extra};add('convert',input,options,lib.convert(input,options));
  }
 }
}
const inputs = {
 hapin: focused,
 arabic: ['','ا ب ت ج ح د ر ز س ش ع ف ق ك ل م ن و ى ي پ چ ڭ گ ھ ۆ ۇ ۋ ە','نع نگ گح سح','ٵٶٷٸ','ٴا ٴو ٴۇ ٴى','ٴبا','ءٴ','ء ','ء\nا','ء"a"ا','ء\u200bا','ءyا','ء😀ا','ء\u0301ا','ءٵا','جاقسى ، ەرلان','جاقسى! ەرلان','  جاقسى  ','ەرلان "w"','"abc','"a\\"b"','😀y،؛؟ـ','Иa'],
 cyrillic: ['','АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЫЭЮЯЁІҒҚҢҮҰҺӘӨъь','нғ ңх нг гх сх','арзанға','күн сый','сый күн','ки ё','ки Ё','ки "w"','ә ө ү і','иъ Эь','  а  б\tв\nг','"abc','"a\\"b"','😀y,;?-','г\u200bх','ңх нғ нг'],
};
const wrappers = [
 ['transformHapinToArabic','hapin','arabic'], ['transformHapinToIPA_CNA','hapin','ipa-cna'],
 ['transformArabicToHapin','arabic','hapin'], ['transformArabicToIPA_CNA','arabic','ipa-cna'],
 ['transformCyrillicToHapin','cyrillic','hapin'], ['transformCyrillicToArabic','cyrillic','arabic'], ['transformCyrillicToIPA_CNA','cyrillic','ipa-cna'],
];
for (const [from,to] of directions.filter(([s]) => s !== 'hapin')) {
 for (const input of inputs[from]) for (const extra of [{},{rules:'legacy'},{errors:'strict'},{formatting:'legacy'},{inputFormat:'legacy-boundaries'}]) {
  const options={from,to,...extra};add('convert',input,options,lib.convert(input,options));
 }
}
for (const [name,from] of wrappers) for (const input of inputs[from]) for (const clean of [true,false]) for (const easy of (name==='transformArabicToHapin'?[true,false]:[false])) {
 add('legacy',input,{name,clean,easy},lib[name](input,clean,easy));
}
for (const [name,from] of [['transformHapinToArabicArray','hapin'],['transformArabicToHapinArray','arabic'],['transformCyrillicToHapinArray','cyrillic']]) for (const input of inputs[from]) add('array',input,{name},lib[name](input));
for (const input of ['ngh','angye','ashat','shsh hh tsh','ngh ngh ngh']) {
 const analysis=lib.analyzeHapin(input);
 for (let i=0;i<analysis.alternatives.length;i++) {
  const selected={...analysis,units:analysis.alternatives[i].units};
  for (const to of ['arabic','ipa-cna']) for (const errors of ['preserve','strict']) {
   const options={from:'hapin',to,errors};add('analysis-convert',selected,options,lib.convert(selected,options));
  }
 }
}
for (const [input,options] of [['ara',{from:'hapin',to:'hapin'}],['ا',{from:'arabic',to:'arabic'}]]) add('convert',input,options,lib.convert(input,options));
const mismatch=lib.analyzeHapin('ara');add('analysis-convert',mismatch,{from:'arabic',to:'hapin'},lib.convert(mismatch,{from:'arabic',to:'hapin'}));
add('rules','',{},lib.segmentationRules);
fs.mkdirSync(path.join(root,'tests/fixtures'),{recursive:true});
fs.writeFileSync(path.join(root,'tests/fixtures/parity.jsonl'),rows.map(row=>JSON.stringify(row)).join('\n')+'\n');
// Seeded differential cases keep the committed corpus compact while covering many combinations.
let seed=0x48415049;
function rand(n){seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return (seed>>>0)%n;}
const alphabets={
 hapin: 'a b t j h d r z s sh gh f q k l m n o e i p ch ng g hh v u w ye xa xo xe xu shsh ts iw ia io x yu yi yo yw ya c y ngh ngul ngye ngxe tsh'.split(' '),
 arabic: [...'ابتجحدرزسشعفقكلمنوىيپچڭگھۆۇۋەٴءٵٶٷٸ'],
 cyrillic: [...'абвгдеёжзийклмнопрстуфхцчшщыэюяіғқңүұһәөъь'],
};
const mixed=[' ','  ','\t','\r\n','"abc"','"\\\""','"unfinished','😀','y','\u200b',"'",',','؛','!','\u0301','\u0085','\ufeff','İ','K','\u{1d400}'];
const differential=[];
for(let i=0;i<1600;i++) {
 const [from,to]=directions[i%directions.length];
 const tokens=[...alphabets[from],...mixed];let input='';
 for(let j=0,n=1+rand(24);j<n;j++)input+=tokens[rand(tokens.length)];
 const options={from,to,rules:i%3?'context-v1':'legacy',errors:i%5?'preserve':'strict',formatting:i%4?'preserve':'legacy',inputFormat:i%7?'plain':'legacy-boundaries',maxCandidates:1+rand(12)};
 const result=lib.convert(input,options);
 const wrapper=wrappers.find(w=>w[1]===from&&w[2]===to)[0];
 differential.push({input,options,text:result.text,status:result.status,codes:result.diagnostics.map(d=>d.code),losses:result.losses.map(d=>d.code),legacyClean:lib[wrapper](input,true),legacySeparated:lib[wrapper](input,false)});
}
fs.writeFileSync(path.join(root,'tests/fixtures/differential.jsonl'),differential.map(row=>JSON.stringify(row)).join('\n')+'\n');
const metadata={sourcePackage:'hapin-utils',sourceVersion:pkg.version,sourceCommit,structuredCases:rows.length,differentialCases:differential.length,seed:'0x48415049'};
fs.writeFileSync(path.join(root,'tests/fixtures/metadata.json'),JSON.stringify(metadata,null,2)+'\n');
console.log(metadata);
