export const MAX_INPUT_BYTES = 16 * 1024 * 1024;

export const rules = {
  "repeated-phrase": { label: "Repeated phrase", plural: "Repeated phrases", why: "A phrase of four to six words appears at least three times. Check whether it is a deliberate refrain or a habit." },
  "repeated-starter": { label: "Repeated opening", plural: "Repeated openings", why: "Several narration sentences begin the same way. Repetition can create emphasis; keep it when the rhythm is intentional." },
  echo: { label: "Word echo", plural: "Word echoes", why: "A content word returns within 50 words. Names and common function words are excluded; close repetition can still be intentional." },
  nominal: { label: "Wordy construction", plural: "Wordy constructions", why: "A verb and noun may have a shorter verb equivalent. Consider the alternative only if it fits the voice and meaning." },
  "tense-drift": { label: "Possible tense shift", plural: "Tense shifts", why: "At least three classified narration sentences differ from a strong manuscript-wide tense baseline. Flashbacks and deliberate shifts may be right." },
  "pov-drift": { label: "Possible person shift", plural: "Person shifts", why: "Narration pronouns change person for at least three classified sentences. This is a heuristic cue, not a judgment about viewpoint or head-hopping." },
  "dialogue-tag": { label: "Expressive dialogue tag", plural: "Dialogue tags", why: "An expressive verb is used near dialogue. Consider whether the spoken words and surrounding action already convey the delivery." },
  "adverbial-tag": { label: "Modified dialogue tag", plural: "Modified tags", why: "An adverb modifies the nearby speech tag. It may add useful meaning, or repeat what the line already communicates." },
  cliche: { label: "Familiar phrase", plural: "Familiar phrases", why: "This matches a phrase list. Literal uses, character speech, and deliberate familiar expressions can be worth keeping." },
  "long-sentence": { label: "Long sentence", plural: "Long sentences", why: "This sentence exceeds 40 words. Read it for clarity and breath; length alone is not a problem." },
  monotony: { label: "Similar sentence lengths", plural: "Sentence rhythm", why: "At least four narration sentences stay within three words of the same length. A measured rhythm can be deliberate." },
  passive: { label: "Possible passive", plural: "Passive constructions", why: "A form of “be” is followed by a possible participle. This lightweight pattern can also match a description of a state." },
  filter: { label: "Perception or thought", plural: "Perception and thought", why: "A perception or thought word may put distance between the reader and the scene. Keep it when the act of noticing or thinking matters." },
  hedge: { label: "Hedge or intensifier", plural: "Hedges", why: "This word may soften or amplify a statement. Check whether that nuance is useful, particularly in a character’s speech." },
  adverb: { label: "Adverb", plural: "Adverbs", why: "An “-ly” adverb may duplicate the verb or add a useful distinction. Decide from the sentence, not the word alone." },
};

export const groups = [
  { id: "repetition", label: "Repetition", color: "#9bbfff", rules: ["echo", "repeated-phrase", "repeated-starter"] },
  { id: "style", label: "Style", color: "#e2ba75", rules: ["nominal", "cliche", "passive", "filter", "hedge", "adverb"] },
  { id: "rhythm", label: "Rhythm", color: "#7dcfb2", rules: ["long-sentence", "monotony"] },
  { id: "dialogue", label: "Dialogue", color: "#e3a3c5", rules: ["dialogue-tag", "adverbial-tag"] },
  { id: "narrative", label: "Narrative", color: "#c5a4f4", rules: ["tense-drift", "pov-drift"] },
];

export function groupFor(rule) {
  return groups.find((group) => group.rules.includes(rule));
}
