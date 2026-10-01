// Smoke test for the Node binding, run via `npm test` (node --test).
// Requires the native addon: `npm run build:debug` first.

const test = require('node:test');
const assert = require('node:assert');

let addon;
try {
  addon = require('../index.js');
} catch {
  addon = null;
}
const skip = !addon && 'native addon not built -- run `npm run build:debug` first';

const prose =
  '# One\n\n' +
  '"We leave at dawn," Mara said softly. She really felt that the bridge was gone.\n' +
  '"And if the bridge is gone?" Tomas exclaimed.\n' +
  '"Then we swim." Mara shouldered the pack. The river was loud below them. ' +
  'It had rained for three days and the water was brown and fast. The water was very cold.\n\n' +
  '# Two\n\n' +
  'The decision was made by the council at the end of the day. ' +
  'The river was crossed. The river was wide. The river was deep.\n';

test('checkStyle locates adverbs, hedges, filter words, and passives', { skip }, () => {
  const r = addon.checkStyle(prose);
  assert.ok(r.words > 40);
  assert.ok(r.counts.adverbs >= 1, 'softly');
  assert.ok(r.counts.hedges >= 1, 'really/very');
  assert.ok(r.counts.filterWords >= 1, 'felt');
  assert.ok(r.counts.passive >= 1, 'was made');
  assert.strictEqual(r.findings.length, Object.values(r.counts).reduce((a, b) => a + b, 0));
  const f = r.findings[0];
  assert.ok(f.location.line >= 1 && f.location.column >= 1);
  assert.strictEqual(typeof f.location.excerpt, 'string');
  assert.ok(f.location.offset + f.location.length <= Buffer.byteLength(prose));
  assert.strictEqual(r.perThousand.adverbs, (r.counts.adverbs * 1000) / r.words);
});

test('checkStyle honours plain format and custom tics', { skip }, () => {
  assert.throws(() => addon.checkStyle('x', { format: 'rtf' }), /unknown format/);
  const plain = addon.checkStyle('# Not a heading really\n', { format: 'plain', tics: ['heading'] });
  assert.ok(plain.counts.tics >= 1);
  assert.ok(plain.findings.some((f) => f.rule === 'tic' && f.message === 'heading'));
});

test('findEchoes reports the same word reappearing within the window', { skip }, () => {
  const echoes = addon.findEchoes(prose, { window: 30, minLength: 5 });
  assert.ok(echoes.some((e) => e.word === 'river' || e.word === 'water' || e.word === 'bridge'));
  const e = echoes[0];
  assert.ok(e.second.offset > e.first.offset);
  assert.ok(e.distance > 0);
});

test('findRepeats returns recurring phrases with every location', { skip }, () => {
  const repeats = addon.findRepeats(prose, { minWords: 3, maxWords: 3, minCount: 2 });
  const hit = repeats.find((r) => r.phrase === 'the river was');
  assert.ok(hit, JSON.stringify(repeats));
  assert.strictEqual(hit.locations.length, hit.count);
});

test('analyzeRhythm summarises sentence lengths', { skip }, () => {
  const r = addon.analyzeRhythm(prose, { long: 10 });
  assert.strictEqual(r.lengths.length, r.summary.count);
  assert.ok(r.findings.some((f) => f.rule === 'long-sentence'));
  assert.strictEqual(typeof r.variation, 'number');
});

test('sentenceStarters flags runs opening the same way', { skip }, () => {
  const s = addon.sentenceStarters(prose, { run: 3 });
  assert.ok(s.sentence.some((w) => w.word === 'the'));
  assert.ok(s.findings.some((f) => f.rule === 'repeated-starter'));
});

test('analyzeDialogue classifies tags and measures the ratio', { skip }, () => {
  const d = addon.analyzeDialogue(prose);
  assert.ok(d.ratio.dialogueWords > 0 && d.ratio.ratio > 0 && d.ratio.ratio < 1);
  assert.ok(d.attributions.length >= 3);
  assert.ok(d.plainTags >= 1, 'said');
  assert.ok(d.showyTags >= 1, 'exclaimed');
  assert.ok(d.adverbTags >= 1, 'said softly');
  assert.strictEqual(d.plainTags + d.showyTags + d.untagged, d.attributions.length);
  assert.ok(d.bySection.length >= 1);
});

test('sectionArc profiles each heading', { skip }, () => {
  const arc = addon.sectionArc(prose);
  assert.deepStrictEqual(arc.map((s) => s.section), ['One', 'Two']);
  assert.ok(arc[0].dialogueRatio > arc[1].dialogueRatio);
});

test('report aggregates everything with sane shapes', { skip }, () => {
  const r = addon.report(prose, { top: 5 });
  assert.strictEqual(r.counts.words, r.readability.words);
  assert.ok(r.readability.fleschKincaidGrade > 0);
  assert.ok(r.diversity.unique <= r.diversity.words);
  assert.ok(r.topWords.length <= 5);
  assert.ok(r.sentenceLengths.max >= r.sentenceLengths.min);
  assert.strictEqual(r.style.adverbs, addon.checkStyle(prose).counts.adverbs);
});

test('empty input is safe everywhere', { skip }, () => {
  assert.strictEqual(addon.checkStyle('').words, 0);
  assert.deepStrictEqual(addon.findEchoes(''), []);
  assert.deepStrictEqual(addon.findRepeats(''), []);
  assert.deepStrictEqual(addon.sectionArc(''), []);
  assert.strictEqual(addon.analyzeDialogue('').attributions.length, 0);
  assert.strictEqual(addon.report('').counts.words, 0);
});
