/** Removes credentials and identifying home-directory segments from shareable diagnostics. */
export function redact(text: string): string {
  return text
    .replace(/(--(?:accessToken|clientToken|session)\s+)(?:"[^"]*"|'[^']*'|\S+)/gi, "$1<redacted>")
    .replace(/\/home\/[^/\s]+(?:\/|\\)/g, "~/")
    .replace(/[A-Z]:\\Users\\[^\\\s]+\\/gi, "~/")
    .replace(/\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b/g, "<redacted>");
}
