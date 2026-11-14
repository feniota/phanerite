// Requests that hit a file in the snapshot never reach this script: the platform
// serves matching assets first (run_worker_first is off). Everything left is a miss,
// so the job here is directory spellings and a 404 that explains the URL scheme.

export default {
  async fetch(request, env) {
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return page('Method Not Allowed', '<p>This docs site serves GET and HEAD only.</p>', {
        status: 405,
        allow: 'GET, HEAD',
      });
    }

    const url = new URL(request.url);
    const probe = indexProbe(url.pathname);
    if (probe) {
      const found = await env.ASSETS.fetch(new Request(new URL(probe, url).toString(), request));
      if (found.status === 200) {
        // `/dev/gui` has to become a redirect, not an inline copy of `/dev/gui/index.html`:
        // rustdoc links are relative, so they resolve against the URL the browser keeps.
        if (!url.pathname.endsWith('/')) {
          return Response.redirect(new URL(`${url.pathname}/`, url).toString(), 301);
        }
        return found;
      }
    }

    return notFound(url, env);
  },
};

function indexProbe(pathname) {
  if (pathname === '/') return '/index.html';
  if (pathname.endsWith('/')) return `${pathname}index.html`;
  // A dotted last segment is a file request, not a directory someone forgot to slash.
  return lastSegment(pathname).includes('.') ? null : `${pathname}/index.html`;
}

async function notFound(url, env) {
  const branches = await readBranches(env, url);
  const pathname = decodeSafe(url.pathname);
  const wanted = longestBranch(branches, pathname);

  const intro = wanted
    ? `<p>Branch <code>${esc(wanted.branch)}</code> has published docs, but this path is not one of them.</p>`
    : branches.length
      ? `<p>No branch named <code>${esc(firstSegment(pathname))}</code> has published docs here.</p>`
      : '<p>No branch has published docs yet.</p>';

  const list = wanted
    ? `<p><a href="/${esc(wanted.path)}/">Open ${esc(wanted.branch)} docs</a></p>`
    : `<ul>${branches
        .slice(0, 12)
        .map((entry) => `<li><a href="/${esc(entry.path)}/">${esc(entry.branch)}</a></li>`)
        .join('\n')}</ul>${branches.length > 12 ? '<p>…and more on the <a href="/">branch index</a>.</p>' : ''}`;

  return page('Not Found', `${intro}${branches.length ? list : ''}`, { status: 404 });
}

async function readBranches(env, url) {
  const response = await env.ASSETS.fetch(new Request(new URL('/branches.json', url).toString()));
  if (response.status !== 200) return [];
  const data = await response.json().catch(() => null);
  return Array.isArray(data?.branches) ? data.branches : [];
}

function longestBranch(branches, pathname) {
  const clean = pathname.replace(/^\/+/, '');
  let best;
  let bestLength = -1;
  for (const entry of branches) {
    const rel = decodeSafe(entry.path);
    if ((clean === rel || clean.startsWith(`${rel}/`)) && rel.length > bestLength) {
      best = entry;
      bestLength = rel.length;
    }
  }
  return best;
}

function page(title, bodyHtml, headers = {}) {
  const { status = 200, ...rest } = headers;
  return new Response(
    `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)} &middot; Phanerite docs</title>
<link rel="stylesheet" href="/site.css">
</head>
<body class="notice">
<h1>${esc(title)}</h1>
${bodyHtml}
<p class="home">Phanerite publishes docs per branch, at <code>/&lt;branch&gt;/&#8230;</code> &mdash;
see the <a href="/">branch index</a>.</p>
</body>
</html>
`,
    {
      status,
      headers: {
        'content-type': 'text/html; charset=utf-8',
        'cache-control': 'public, max-age=60',
        'x-content-type-options': 'nosniff',
        ...rest,
      },
    },
  );
}

function lastSegment(pathname) {
  const trimmed = pathname.replace(/\/+$/, '');
  return trimmed.slice(trimmed.lastIndexOf('/') + 1);
}

function firstSegment(pathname) {
  return pathname.split('/').filter(Boolean)[0] ?? '';
}

function decodeSafe(value) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function esc(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}
