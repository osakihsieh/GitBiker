/**
 * Convert a branch name input to a valid git branch name.
 * Rules:
 *  1. Spaces → hyphens
 *  2. Consecutive hyphens → single hyphen
 *  3. Remove characters not in [a-zA-Z0-9._/-]
 *  4. Strip leading/trailing hyphens and dots
 *  5. Preserve case (no lowercase conversion)
 */
export function slugifyBranchName(input: string): string {
  return input
    .replace(/\s+/g, '-')
    .replace(/[^a-zA-Z0-9._/-]/g, '')
    .replace(/-{2,}/g, '-')
    .replace(/^[-.]/, '')
    .replace(/[-.]$/, '');
}
