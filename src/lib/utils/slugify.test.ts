import { describe, it, expect } from 'vitest';
import { slugifyBranchName } from './slugify';

describe('slugifyBranchName', () => {
  it('converts spaces to hyphens', () => {
    expect(slugifyBranchName('my cool feature')).toBe('my-cool-feature');
  });

  it('collapses consecutive hyphens', () => {
    expect(slugifyBranchName('a--b---c')).toBe('a-b-c');
  });

  it('removes invalid characters', () => {
    expect(slugifyBranchName('feat@#$%name')).toBe('featname');
  });

  it('preserves slashes', () => {
    expect(slugifyBranchName('feat/add-thing')).toBe('feat/add-thing');
  });

  it('preserves dots and underscores', () => {
    expect(slugifyBranchName('v1.0_release')).toBe('v1.0_release');
  });

  it('strips leading hyphens and dots', () => {
    expect(slugifyBranchName('-test')).toBe('test');
    expect(slugifyBranchName('.test')).toBe('test');
  });

  it('strips trailing hyphens and dots', () => {
    expect(slugifyBranchName('test-')).toBe('test');
    expect(slugifyBranchName('test.')).toBe('test');
  });

  it('handles multiple spaces as single hyphen', () => {
    expect(slugifyBranchName('a  b  c')).toBe('a-b-c');
  });

  it('returns empty for empty input', () => {
    expect(slugifyBranchName('')).toBe('');
  });

  it('preserves case', () => {
    expect(slugifyBranchName('MyFeature')).toBe('MyFeature');
  });

  it('handles leading dashes from spaces + special chars', () => {
    expect(slugifyBranchName('---test---')).toBe('test');
  });

  it('handles complex real-world input', () => {
    expect(slugifyBranchName('fix: handle (null) values')).toBe('fix-handle-null-values');
  });
});
