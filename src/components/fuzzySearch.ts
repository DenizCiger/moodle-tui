function isBoundaryChar(char: string): boolean {
  return char === " " || char === "-" || char === "_" || char === "/" || char === ".";
}

export function fuzzyScore(queryRaw: string, candidateRaw: string): number | null {
  const query = queryRaw.toLowerCase();
  const candidate = candidateRaw.toLowerCase();
  if (!query) return 0;
  if (!candidate) return null;

  let queryIdx = 0;
  let previousMatchIdx = -1;
  let score = 0;

  for (let idx = 0; idx < candidate.length && queryIdx < query.length; idx += 1) {
    if (candidate[idx] !== query[queryIdx]) continue;

    score += 1;

    if (previousMatchIdx === idx - 1) {
      score += 6;
    }

    const previousChar = idx > 0 ? candidate[idx - 1] || "" : "";
    if (idx === 0 || isBoundaryChar(previousChar)) {
      score += 4;
    }

    if (idx < 6) {
      score += (6 - idx) * 0.25;
    }

    previousMatchIdx = idx;
    queryIdx += 1;
  }

  if (queryIdx !== query.length) {
    return null;
  }

  score -= candidate.length * 0.01;
  return score;
}
