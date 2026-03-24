import type { QaIssue } from "../types";

const TAG_RE = /<\/?(?:x|g|bx|ex|ph|it|mrk|pc|sc|ec|sm|em)\b[^>]*\/?>/gi;
const NUM_RE = /\b\d[\d.,]*\b/g;

function extractTags(text: string): Set<string> {
  return new Set([...text.matchAll(TAG_RE)].map((m) => m[0]));
}

function extractNumbers(text: string): string[] {
  return [...text.matchAll(NUM_RE)].map((m) => m[0]);
}

function checkTags(source: string, target: string): QaIssue[] {
  const srcTags = extractTags(source);
  const tgtTags = extractTags(target);
  const issues: QaIssue[] = [];

  for (const tag of srcTags) {
    if (!tgtTags.has(tag)) {
      issues.push({
        kind: "missing_tag",
        message: `Tag \`${tag}\` is present in source but missing from target.`,
      });
    }
  }

  return issues;
}

function checkNumbers(source: string, target: string): QaIssue[] {
  return extractNumbers(source)
    .filter((n) => !target.includes(n))
    .map((n) => ({
      kind: "missing_number",
      message: `Number \`${n}\` is present in source but missing from target.`,
    }));
}

export function runQaChecks(source: string, target: string): QaIssue[] {
  return [...checkTags(source, target), ...checkNumbers(source, target)];
}
