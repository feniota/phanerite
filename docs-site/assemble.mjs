#!/usr/bin/env node
// Mounts one branch's rustdoc output into the docs-site snapshot and regenerates
// the branch index. The snapshot tree is what `wrangler deploy` publishes, so the
// index is always rebuilt for *every* branch in the tree, not just the caller's.

import { appendFile, cp, mkdir, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';

// The landing page, its data, and the stylesheet they share with the Worker's 404 page
// live at the site root, so they own those names.
const RESERVED_ROOT_NAMES = new Set(['index.html', 'branches.json', 'site.css', 'fonts', 'static.files']);
// rustdoc writes this file at the root of every doc tree; its presence is what marks a
// subtree as "some branch's docs" when the snapshot is rescanned. (static.files used to
// be the marker, but it now lives once at the site root, shared by every branch.)
const DOC_ROOT_MARKER = 'crates.js';
// rustdoc content-hashes every file in here, so the union of all branches' copies is
// collision-free: one shared directory serves every branch and the browser caches each
// file once, no matter how many branches reference it.
const STATIC_DIR = 'static.files';
const SEP = '<span class="sep"> &middot; </span>';
// The faces rustdoc renders its pages with: its body text is Source Serif 4 and its
// headings are Fira Sans. File names are content-hashed per rustdoc release, hence the
// prefix search. Fira Sans ships only Regular and Medium, so bolder headings are
// synthesized from Medium exactly as rustdoc's own pages do it.
const FONT_FACES = [
  { prefix: 'SourceSerif4-Regular-', family: 'Source Serif 4', weight: 400, local: 'Source Serif 4' },
  { prefix: 'SourceSerif4-Semibold-', family: 'Source Serif 4', weight: 500, local: 'Source Serif 4 Semibold' },
  { prefix: 'SourceSerif4-Bold-', family: 'Source Serif 4', weight: 700, local: 'Source Serif 4 Bold' },
  { prefix: 'FiraSans-Regular-', family: 'Fira Sans', weight: 400, local: 'Fira Sans' },
  { prefix: 'FiraSans-Medium-', family: 'Fira Sans', weight: 500, local: 'Fira Sans Medium' },
];

const args = parseArgs(process.argv.slice(2));
for (const name of ['site', 'docs', 'branch', 'commit']) {
  if (!args[name]) die(`--${name} is required`);
}

const siteDir = path.resolve(args.site);
const docsDir = path.resolve(args.docs);
const repoUrl = args.repo ?? '';
const builtAt = new Date().toISOString();

const branchRelPath = branchToPath(args.branch);
if (branchRelPath !== args.branch.trim()) {
  // Realistic branch names never need this; anything that does is worth a glance on
  // the live site, because percent-encoded paths depend on how the CDN decodes URLs.
  console.warn(`assemble: branch "${args.branch}" is not URL-safe, published at /${branchRelPath}/`);
}
if (!(await hasMarker(docsDir))) {
  die(`${docsDir} does not look like rustdoc output (no ${DOC_ROOT_MARKER}); refusing to publish it`);
}
await assertPathUsable(siteDir, branchRelPath, args.branch);

const targetDir = path.join(siteDir, branchRelPath);
await rm(targetDir, { recursive: true, force: true });
await mkdir(path.dirname(targetDir), { recursive: true });
await cp(docsDir, targetDir, { recursive: true, force: true });

await shareStaticFiles(targetDir, siteDir);
await rewriteStaticRefs(targetDir);

const crates = await readCrates(path.join(targetDir, 'crates.js'));
// cargo leaves a lock sentinel in target/doc; it is not part of the documentation.
await rm(path.join(targetDir, '.lock'), { force: true });

const mounted = {
  branch: args.branch,
  path: branchRelPath,
  commit: args.commit,
  builtAt,
  toolchain: args.toolchain ?? '',
  crates,
};
await writeBranchIndex(targetDir, mounted);
// Rewritten from this build's rustdoc every time, so the root pages' faces track whatever
// version the newest branch was documented with.
await publishStylesheet(docsDir, siteDir);

await writeFile(path.join(targetDir, 'build.json'), `${JSON.stringify(mounted, null, 2)}\n`);

const entries = await collectEntries(siteDir);

// Forgejo never runs a workflow for a deleted branch, so the snapshot is cleaned by
// whoever writes it next: docs.yml feeds it the live branch list and stale trees go.
const pruned = args['live-branches-file'] ? await pruneStale(entries) : 0;

entries.sort((a, b) => b.builtAt.localeCompare(a.builtAt) || a.branch.localeCompare(b.branch));

await writeFile(
  path.join(siteDir, 'branches.json'),
  `${JSON.stringify({ generatedAt: builtAt, repo: repoUrl, branches: entries }, null, 2)}\n`,
);
await writeFile(path.join(siteDir, 'index.html'), renderLanding(entries, { repoUrl, builtAt }));
await writeHeaders(siteDir);

console.log(`Mounted ${args.branch} at /${branchRelPath}/ (${crates.length} crates).`);
console.log(
  `Indexed ${entries.length} branch tree${entries.length === 1 ? '' : 's'} from ${siteDir}` +
    (pruned ? `, dropped ${pruned} deleted branch${pruned === 1 ? '' : 'es'}.` : '.'),
);

// docs.yml turns this into a notice linking straight to the branch it just built.
if (process.env.GITHUB_OUTPUT) {
  await appendFile(process.env.GITHUB_OUTPUT, `mounted_path=${branchRelPath}\n`);
}

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (!token.startsWith('--')) die(`unexpected argument: ${token}`);
    const eq = token.indexOf('=');
    if (eq !== -1) {
      out[token.slice(2, eq)] = token.slice(eq + 1);
    } else {
      const next = argv[i + 1];
      if (next === undefined || next.startsWith('--')) die(`--${token.slice(2)} needs a value`);
      out[token.slice(2)] = next;
      i += 1;
    }
  }
  return out;
}

// A branch name is already a `/`-separated path, so `dev/gui` becomes nested folders
// exactly like refs under .git/. Everything else a URL or a filesystem would misread
// is percent-encoded; a browser re-encodes it the same way when it asks for the page.
function branchToPath(name) {
  const trimmed = name.trim();
  if (trimmed === '') die('branch name is empty');
  const segments = trimmed.split('/');
  for (const segment of segments) {
    if (segment === '') die(`branch "${name}" has an empty path segment`);
    if (segment === '.' || segment === '..') die(`branch "${name}" would escape the site root`);
    // eslint-disable-next-line no-control-regex
    if (/[\x00-\x1f\x7f]/.test(segment)) die(`branch "${name}" contains control characters`);
  }
  const encoded = segments.map(encodeURIComponent);
  if (encoded.length === 1 && RESERVED_ROOT_NAMES.has(encoded[0])) {
    die(`branch "${name}" collides with the generated root file /${encoded[0]}`);
  }
  return encoded.join('/');
}

// Git cannot hold a file and a directory with the same name, and mounting a branch
// deletes its old subtree first, so three shapes would corrupt other branches' docs:
// the target already being a file, the path crossing one, or the target containing
// nested branch trees (`dev` mounted while `dev/gui` lives underneath it).
async function assertPathUsable(root, rel, branch) {
  const own = await stat(path.join(root, rel)).catch(() => null);
  if (own && !own.isDirectory()) {
    die(`${rel} is a file in the snapshot, so branch "${branch}" cannot be mounted on top of it`);
  }
  let walked = '';
  for (const segment of rel.split('/')) {
    walked = walked ? `${walked}/${segment}` : segment;
    const info = await stat(path.join(root, walked)).catch(() => null);
    if (info && !info.isDirectory()) {
      die(`${walked} is a file in the snapshot, so branch "${branch}" cannot be mounted at /${rel}/`);
    }
  }
  if (own) {
    const nested = await findNestedRoots(path.join(root, rel), rel);
    if (nested.length) {
      die(
        `branch "${branch}" would delete nested branch trees mounted below it: ${nested
          .slice(0, 3)
          .map((entry) => `/${entry}`)
          .join(', ')}${nested.length > 3 ? ', …' : ''}`,
      );
    }
  }
}

// Returns site-root-relative paths of doc trees found strictly below `absDir`.
async function findNestedRoots(absDir, relPrefix) {
  const found = [];
  const top = await readdir(absDir, { withFileTypes: true }).catch(() => []);
  for (const dirent of top) {
    if (!isBranchDir(dirent)) continue;
    await walk(path.join(absDir, dirent.name), relPrefix ? `${relPrefix}/${dirent.name}` : dirent.name);
  }
  return found;

  async function walk(abs, rel) {
    const dirents = await readdir(abs, { withFileTypes: true }).catch(() => []);
    if (dirents.some((d) => d.name === DOC_ROOT_MARKER && d.isFile())) found.push(rel);
    for (const dirent of dirents) {
      if (!isBranchDir(dirent)) continue;
      await walk(path.join(abs, dirent.name), `${rel}/${dirent.name}`);
    }
  }
}

function isBranchDir(dirent) {
  return dirent.isDirectory() && dirent.name !== '.git' && dirent.name !== 'node_modules';
}

async function hasMarker(dir) {
  const info = await stat(path.join(dir, DOC_ROOT_MARKER)).catch(() => null);
  return Boolean(info?.isFile());
}

// The element shape of rustdoc's crate index is nightly-dependent, and every branch is
// documented with the channel pinned in its own rust-toolchain.toml, so both occur:
//   nightly-2026-08-25  window.ALL_CRATES = ["phanerite_core"];
//   nightly-2026-11-13  window.ALL_CRATES = [{"c":"phanerite_core"}];
async function readCrates(cratesJsPath) {
  const text = await readFile(cratesJsPath, 'utf8').catch(() => '');
  const list = text.match(/ALL_CRATES\s*=\s*\[([^\]]*)\]/);
  if (!list) return [];
  const body = list[1].trim();
  // Read whichever of these two shapes the array turns out to be, and nothing at all if
  // it is neither: a guessed crate name would publish a dead link.
  if (body.startsWith('{')) return [...body.matchAll(/"c":"([^"]+)"/g)].map((found) => found[1]);
  if (body.startsWith('"')) return [...body.matchAll(/"([^"]*)"/g)].map((found) => found[1]);
  return [];
}

// cargo's own root index.html is an unstyled list, and it only appears when a workspace
// documents several crates. Either way the branch entry point gets the landing page's
// design, so /<branch>/ looks like the rest of the site and not like bare HTML.
async function writeBranchIndex(dir, meta) {
  if (!meta.crates.length) {
    // Nothing to list, which is what a nightly that changes crates.js again would give us.
    // Then cargo's own index is better than a failed deploy, but only if it exists.
    if (await stat(path.join(dir, 'index.html')).then((info) => info.isFile(), () => false)) return;
    die(`branch "${meta.branch}" has no crate list and no index.html to serve`);
  }
  const rows = [];
  for (const name of meta.crates) {
    const summary = await readCrateSummary(path.join(dir, name, 'index.html'));
    rows.push(
      `<li><a class="branch" href="${esc(name)}/index.html">${esc(name)}</a>${
        summary ? `\n<span class="meta">${summary}</span>` : ''
      }</li>`,
    );
  }
  // rustdoc puts a Source link under the crate heading; here it points at the exact commit
  // this tree was built from, which is the one fact a branch page owes the reader.
  const source =
    meta.commit && repoUrl
      ? `<p class="source"><a href="${esc(`${repoUrl}/commit/${meta.commit}`)}">Source</a></p>\n`
      : '';
  const foot = [`Built ${formatDate(meta.builtAt)}`, meta.toolchain || null].filter(Boolean);
  const title = `Branch ${esc(meta.branch)} &middot; Phanerite docs`;
  await writeFile(
    path.join(dir, 'index.html'),
    `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title}</title>
<link rel="stylesheet" href="/site.css">
</head>
<body class="index">
<h1>Branch ${esc(meta.branch)}</h1>
${source}<ul>
${rows.join('\n')}
</ul>
<footer>
<p><a href="/">All branches</a><span class="sep"> &middot; </span>${
      repoUrl ? `<a href="${esc(repoUrl)}">Repository</a>` : 'Repository'
    }</p>
<p>${foot.join(SEP)}</p>
</footer>
</body>
</html>
`,
  );
}

// rustdoc puts a crate's opening paragraph in that crate's own <meta> tag, which is the
// cheapest way to get a summary without parsing the page body. Escaped back to text
// first so it can be escaped once more, safely, as element content.
async function readCrateSummary(indexHtmlPath) {
  const head = (await readFile(indexHtmlPath, 'utf8').catch(() => '')).slice(0, 4096);
  const described = head.match(/<meta name="description" content="([^"]*)"/);
  if (!described || !described[1].trim()) return '';
  const text = described[1]
    .replaceAll('&quot;', '"')
    .replaceAll('&#39;', "'")
    .replaceAll('&lt;', '<')
    .replaceAll('&gt;', '>')
    .replaceAll('&amp;', '&');
  return esc(text);
}

// Doc roots are the only things removed, and only when their branch is gone from the
// source repository; `entries[].path` always comes from the filesystem walk, never
// from a stored file, so nothing outside the snapshot can be named here.
async function pruneStale(entries) {
  const live = new Set(
    (await readFile(args['live-branches-file'], 'utf8'))
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean),
  );
  if (live.size === 0) die('the live branch list is empty; refusing to prune every branch');
  // Ours is published right now, whatever the list says.
  live.add(args.branch);
  const stale = entries.filter((entry) => !live.has(entry.branch));
  for (const entry of stale) {
    await rm(path.join(siteDir, entry.path), { recursive: true, force: true });
  }
  for (let i = entries.length - 1; i >= 0; i -= 1) {
    if (!live.has(entries[i].branch)) entries.splice(i, 1);
  }
  return stale.length;
}

async function collectEntries(root) {
  const found = [];
  await walk('');
  return found;

  async function walk(rel) {
    let dirents;
    try {
      dirents = await readdir(path.join(root, rel), { withFileTypes: true });
    } catch {
      return;
    }
    if (rel && dirents.some((d) => d.name === DOC_ROOT_MARKER && d.isFile())) {
      found.push(await describeTree(root, rel));
    }
    for (const dirent of dirents) {
      if (!isBranchDir(dirent)) continue;
      await walk(rel ? `${rel}/${dirent.name}` : dirent.name);
    }
  }
}

// A tree with no readable build.json (a branch published before this metadata
// existed, or one assembled by hand) still gets listed from its own timestamps.
async function describeTree(root, rel) {
  const fallback = { branch: pathToBranch(rel), path: rel, commit: '', builtAt: '', toolchain: '', crates: [] };
  const stored = await readFile(path.join(root, rel, 'build.json'), 'utf8')
    .then((text) => JSON.parse(text))
    .catch(() => null);
  if (stored && typeof stored.builtAt === 'string' && stored.builtAt) {
    return { ...fallback, ...stored, path: rel, branch: stored.branch ?? fallback.branch };
  }
  const mtime = await stat(path.join(root, rel, 'index.html'))
    .then((info) => info.mtime.toISOString())
    .catch(() => new Date(0).toISOString());
  return { ...fallback, builtAt: mtime, crates: await readCrates(path.join(root, rel, 'crates.js')) };
}

function pathToBranch(rel) {
  return rel.split('/').map(decodeSegment).join('/');
}

function decodeSegment(segment) {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

// Moves this branch's static.files into the shared site-root union and drops the local
// copy. Names are content hashes, so an existing file is either byte-identical (keep it)
// or a hash collision (stop loudly rather than serve one branch's bytes as another's).
// The union is append-only: a pruned branch's files linger, but they are few and tiny and
// still serve any reader holding an older page.
async function shareStaticFiles(targetDir, siteDir) {
  const shared = path.join(siteDir, STATIC_DIR);
  await mkdir(shared, { recursive: true });
  const source = path.join(targetDir, STATIC_DIR);
  for (const name of await readdir(source)) {
    const to = path.join(shared, name);
    const existing = await readFile(to).catch(() => null);
    if (existing) {
      if (!existing.equals(await readFile(path.join(source, name)))) {
        die(`${STATIC_DIR}/${name} already exists with different bytes; rustdoc's content hash collided`);
      }
      continue;
    }
    await cp(path.join(source, name), to);
  }
  await rm(path.join(targetDir, STATIC_DIR), { recursive: true, force: true });
}

// rustdoc addresses its assets as ../static.files/… relative to each page, and the
// lazy-loaded scripts resolve against data-static-root-path, which carries the same
// prefix. Rewriting every occurrence to the shared root path is what lets one cached
// copy serve all branches; the relative data-root-path (crate links, search index) is
// left alone because those files really are per-branch.
async function rewriteStaticRefs(dir) {
  for (const dirent of await readdir(dir, { withFileTypes: true })) {
    const abs = path.join(dir, dirent.name);
    if (dirent.isDirectory()) {
      await rewriteStaticRefs(abs);
    } else if (dirent.name.endsWith('.html')) {
      const html = await readFile(abs, 'utf8');
      const rewritten = html.replace(/(?:\.\.\/)+static\.files\//g, `/${STATIC_DIR}/`);
      if (rewritten !== html) await writeFile(abs, rewritten);
    }
  }
}

// Workers Assets serves everything with `max-age=0, must-revalidate` by default, which
// revalidates every font and script on each navigation. Content-hashed names never change
// meaning, so they can be cached for a year; HTML and the generated root files keep the
// revalidating default so a deploy is visible immediately. Rules are per hashed file (the
// platform caps _headers at 100 rules), so a non-hashed file an older rustdoc might ship
// keeps the safe default instead of being frozen for a year.
async function writeHeaders(siteDir) {
  const shared = await readdir(path.join(siteDir, STATIC_DIR)).catch(() => []);
  const rules = shared
    .filter((name) => /-[0-9a-f]{8}\./.test(name))
    .map((name) => `/${STATIC_DIR}/${name}\n  Cache-Control: public, max-age=31536000, immutable\n`);
  await writeFile(path.join(siteDir, '_headers'), rules.join(''));
}

// The Worker's 404 page links this same file, so the palette and the faces live once.
// The faces are the shared /static.files/ copies, so the site's own pages and the rustdoc
// pages fetch the exact same font URLs and share one browser cache.
async function publishStylesheet(docsDir, siteDir) {
  const available = await readdir(path.join(docsDir, STATIC_DIR)).catch(() => []);
  const faces = [];
  for (const face of FONT_FACES) {
    const name = available.find((entry) => entry.startsWith(face.prefix) && entry.endsWith('.woff2'));
    if (!name) continue;
    faces.push({ ...face, name });
  }
  // The site used to keep its own copy of the faces here; drop it so the snapshot does
  // not carry the duplicates forward.
  await rm(path.join(siteDir, 'fonts'), { recursive: true, force: true });
  await writeFile(path.join(siteDir, 'site.css'), renderStylesheet(faces));
}

function renderStylesheet(faces) {
  const fontFaces = faces
    .map(
      (face) =>
        `@font-face {\n  font-family: "${face.family}";\n  font-style: normal;\n  font-display: swap;\n  font-weight: ${face.weight};\n  src: local("${face.local}"), url("/${STATIC_DIR}/${face.name}") format("woff2");\n}`,
    )
    .join('\n');
  // The stacks are rustdoc's own, so the name in front is useful even when its face is
  // missing from this build: local() still matches a font the reader has installed.
  const palette = `${fontFaces ? `${fontFaces}\n` : ''}
:root { color-scheme: light dark; --fg: #1b1b1b; --muted: #6a6a6a; --line: #e0e0e0; --accent: #3873ad;
  --serif: "Source Serif 4", "NanumBarunGothic", Georgia, serif;
  --sans: "Fira Sans", sans-serif; }
@media (prefers-color-scheme: dark) {
  :root { --fg: #e6e6e6; --muted: #9d9d9d; --line: #666666; --accent: #d2991d; }
  body { background: #353535; }
}
body { margin: 0 auto; padding: 3rem 1.25rem 4rem; max-width: 48rem; color: var(--fg);
  font-size: 1rem; line-height: 1.5; font-family: var(--serif); }
h1, .branch { margin-block: 0; font-family: var(--sans); }
h1 { font-weight: 500; letter-spacing: -.01em; }
.source { margin: 0 0 1.75rem; font-family: var(--sans); font-size: 1rem; font-weight: 400; }
a { color: var(--accent); text-decoration: none; }
a:hover, a:focus { text-decoration: underline; }
code { font-family: "Source Code Pro", ui-monospace, SFMono-Regular, Menlo, monospace; font-size: .92em; }

.index h1 { font-size: 1.6rem; }
.index ul { list-style: none; margin: 0; padding: 0; }
.index li { padding: .85rem .15rem; border-bottom: 1px solid var(--line); }
.index li.empty { color: var(--muted); border-bottom: 0; }
.index .branch { font-weight: 500; font-size: 1.05rem; }
.index .meta, .index .crates { display: block; color: var(--muted); font-size: .85rem; }
.index .meta { margin-top: .2rem; }
.index footer { margin-top: 2.5rem; color: var(--muted); font-size: .85rem; }

body.notice { padding: 4rem 1.25rem; max-width: 38rem; }
.notice h1 { font-size: 1.4rem; margin-bottom: .75rem; }
.notice ul { padding-left: 1.1rem; }
.notice .home { margin-top: 2rem; color: var(--muted); font-size: .9rem; }
`;
  return `${palette.trimStart()}\n`;
}

function renderLanding(entries, { repoUrl, builtAt }) {
  const rows = entries.length
    ? entries.map(renderRow).join('\n')
    : '<li class="empty">No branch has published docs yet.</li>';
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Project Phanerite</title>
<link rel="stylesheet" href="/site.css">
</head>
<body class="index">
<h1>Project Phanerite</h1>
${repoUrl ? `<p class="source"><a href="${esc(repoUrl)}">Source</a></p>\n` : ''}<ul>
${rows}
</ul>
<footer>
<p>Last updated: ${esc(formatDate(builtAt))}</p>
</footer>
</body>
</html>
`;

  function renderRow(item) {
    const meta = [formatDate(item.builtAt), linkifyCommit(item.commit), item.toolchain || null, plural(item.crates.length, 'crate')];
    return `<li><a class="branch" href="./${esc(item.path)}/">${esc(item.branch)}</a>
<span class="meta">${meta.filter(Boolean).join(SEP)}</span>
<span class="crates">${item.crates.map(esc).join(', ')}</span></li>`;
  }
}

function linkifyCommit(commit) {
  if (!commit) return null;
  const short = esc(commit.slice(0, 7));
  return repoUrl ? `<a href="${esc(`${repoUrl}/commit/${commit}`)}">${short}</a>` : short;
}

function formatDate(iso) {
  return Number.isNaN(Date.parse(iso)) ? iso : `${iso.slice(0, 10)} ${iso.slice(11, 16)} UTC`;
}

function plural(count, noun) {
  return `${count} ${noun}${count === 1 ? '' : 's'}`;
}

function esc(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function die(message) {
  console.error(`assemble: ${message}`);
  process.exit(2);
}
