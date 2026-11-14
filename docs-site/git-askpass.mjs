#!/usr/bin/env node
// git execs this through GIT_ASKPASS (see docs.yml) so the snapshot push token never
// lands in a remote URL, a reflog entry, or the job log.

const prompt = process.argv[2] ?? '';

const answer = /sername/.test(prompt)
  ? (process.env.FORGEJO_ASKPASS_USERNAME ?? 'x-access-token')
  : /assword/.test(prompt)
    ? process.env.FORGEJO_PHANERITE_DOCS_TOKEN
    : undefined;

if (!answer) {
  console.error(`git-askpass: no credential configured for prompt ${JSON.stringify(prompt)}`);
  process.exit(1);
}

process.stdout.write(`${answer}\n`);
