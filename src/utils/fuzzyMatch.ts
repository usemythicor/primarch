export interface FuzzyResult {
  score: number;
  indices: number[];
}

export function fuzzyMatch(query: string, target: string): FuzzyResult | null {
  const q = query.toLowerCase();
  const t = target.toLowerCase();

  if (q.length === 0) return { score: 0, indices: [] };
  if (q.length > t.length) return null;

  const indices: number[] = [];
  let score = 0;
  let qi = 0;
  let lastMatchIndex = -1;

  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      indices.push(ti);

      // Consecutive match bonus
      if (ti === lastMatchIndex + 1) {
        score += 10;
      }

      // Word boundary bonus (after /, -, _, space, or start)
      if (ti === 0 || '/\\-_ '.includes(t[ti - 1])) {
        score += 8;
      }

      // Exact case match bonus
      if (target[ti] === query[qi]) {
        score += 1;
      }

      score += 1; // Base match score
      lastMatchIndex = ti;
      qi++;
    }
  }

  // All query chars must match
  if (qi < q.length) return null;

  // Bonus for shorter targets (more relevant)
  score += Math.max(0, 20 - (t.length - q.length));

  return { score, indices };
}
