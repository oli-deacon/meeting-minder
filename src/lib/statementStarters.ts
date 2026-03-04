export const PROMPTS_PER_SESSION = 5;

export const STATEMENT_STARTERS: readonly string[] = [
  "I want to hear from someone we have not heard from yet.",
  "What perspective might we be missing on this decision?",
  "Can someone challenge the current direction so we can test it?",
  "What risks are we not naming yet?",
  "Lets hear from someone who has not spoken yet?",
  "What is your first reaction to what you are seeing here?",
  "If you had to challenge this idea, what would you say?",
  "What assumptions are we making that we should validate?",
  "Where might this thinking be incomplete?",
  "What tradeoff are we accepting, and is it explicit?",
  "What are the counterpoints we should consider?",
  "What is the smallest experiment we could run next?",
  "Can someone offer an alternative path with different constraints?",
  "What dependencies could block us later?",
  "If this fails, what is most likely to fail first?",
  "What data would increase your confidence in this call?",
  "Who needs to be informed that has not been represented here?",
  "What concern might someone junior feel hesitant to raise?",
  "I am curious how others in the room are thinking about this?",
  "What would we do differently if we had to decide today?"
];

export function pickSessionStarters(source: readonly string[], count: number): string[] {
  const items = [...source];

  for (let i = items.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [items[i], items[j]] = [items[j], items[i]];
  }

  return items.slice(0, Math.min(count, items.length));
}
