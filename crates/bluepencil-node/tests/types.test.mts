// Type-only check of the generated index.d.ts, run via `npm run test:types`.

import {
  checkStyle,
  findEchoes,
  findRepeats,
  analyzeRhythm,
  analyzeDialogue,
  sectionArc,
  sentenceStarters,
  report,
  type StyleReport,
  type Echo,
  type Report,
} from '../index.js';

const style: StyleReport = checkStyle('Example.', { format: 'plain', tics: ['example'] });
const rule: string = style.findings[0]?.rule ?? '';
const echoes: Echo[] = findEchoes('Example.', { window: 20 });
const line: number = echoes[0]?.first.line ?? 0;
const repeats = findRepeats('Example.');
const count: number = repeats[0]?.count ?? 0;
const variation: number = analyzeRhythm('Example.').variation;
const ratio: number = analyzeDialogue('Example.').ratio.ratio;
const sections: string[] = sectionArc('Example.').map((s) => s.section);
const starters: string[] = sentenceStarters('Example.').sentence.map((w) => w.word);
const full: Report = report('Example.');
const grade: number = full.readability.fleschKincaidGrade;
void [rule, line, count, variation, ratio, sections, starters, grade];
