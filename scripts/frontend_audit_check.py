#!/usr/bin/env python3
"""frontend_audit_check.py <npm audit json> <allow list json> [--today YYYY-MM-DD] [--patched <json>]

The all-scope npm audit of a frontend (dev dependencies included), checked against a dated allow list. The production
scope is gated separately and strictly (`npm audit --package-lock-only --omit=dev --audit-level=low`); this check is
for build-time tools whose advisories may have no patched release yet.

The audit is `npm audit --package-lock-only --json` (npm 10 or 11). Its advisories are the objects in each
vulnerability's "via" list: (GHSA id from the url, package name). Packages only reached through another advisory carry
no advisory of their own.

The allow list:
  {"entries": [{"advisory": "GHSA-....", "package": "braces", "reason": "...", "added": "YYYY-MM-DD",
                "expires": "YYYY-MM-DD", "run": "where the entry was added (audit run)"}]}

Fails (exit 1, every finding printed) on:
  - an advisory in the audit that no entry lists (package and id both match)
  - a listed advisory that now has a first patched version (GitHub advisory database): fix it, don't keep it listed
  - an expired entry (today > expires), or one whose expiry is more than 90 days after its date
  - an entry that matches no advisory in the audit (stale)
  - a malformed file (not the shape above, a duplicate entry, an empty reason or run, a bad date)
Exit 2 when the audit compared 0 packages or cannot be read (a vacuous pass is not a pass), or when the advisory
database cannot be asked. Exit 0 prints the counts.

--patched takes a JSON object {GHSA id: first patched version or null} instead of asking the GitHub API (tests).
"""
from __future__ import annotations

import datetime as dt
import json
import os
import re
import sys
import urllib.request

MAX_DAYS = 90
GHSA = re.compile(r"GHSA-[0-9a-z]{4}-[0-9a-z]{4}-[0-9a-z]{4}")
FIELDS = ("advisory", "package", "reason", "added", "expires", "run")


class Bad(Exception):
    pass


def advisories(audit: dict) -> set[tuple[str, str]]:
    out = set()
    for name, v in (audit.get("vulnerabilities") or {}).items():
        for via in v.get("via") or []:
            if isinstance(via, dict):
                m = GHSA.search(via.get("url") or "")
                if not m:
                    raise Bad(f"audit: advisory of {name} has no GHSA id ({via.get('url')!r})")
                out.add((m.group(0), via.get("name") or name))
    return out


def compared(audit: dict) -> int:
    deps = (audit.get("metadata") or {}).get("dependencies")
    if isinstance(deps, dict):
        return int(deps.get("total") or 0)
    return int(deps or 0)


def date(s: object, what: str) -> dt.date:
    if not isinstance(s, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", s):
        raise Bad(f"{what}: {s!r} is not YYYY-MM-DD")
    try:
        return dt.date.fromisoformat(s)
    except ValueError as e:
        raise Bad(f"{what}: {e}") from None


def entries(allow: object) -> list[dict]:
    if not isinstance(allow, dict) or set(allow) != {"entries"} or not isinstance(allow["entries"], list):
        raise Bad('allow list: must be {"entries": [...]}')
    seen = set()
    for i, e in enumerate(allow["entries"]):
        if not isinstance(e, dict) or set(e) != set(FIELDS):
            raise Bad(f"allow list entry {i}: fields must be exactly {', '.join(FIELDS)}")
        if not isinstance(e["advisory"], str) or not GHSA.fullmatch(e["advisory"]):
            raise Bad(f"allow list entry {i}: advisory {e['advisory']!r} is not a GHSA id")
        for k in ("package", "reason", "run"):
            if not isinstance(e[k], str) or not e[k].strip():
                raise Bad(f"allow list entry {i}: {k} is empty")
        a, x = date(e["added"], f"entry {i} added"), date(e["expires"], f"entry {i} expires")
        if x < a:
            raise Bad(f"allow list entry {i}: expires before it was added")
        key = (e["advisory"], e["package"])
        if key in seen:
            raise Bad(f"allow list entry {i}: {key} listed twice")
        seen.add(key)
    return allow["entries"]


def first_patched(ids: list[str]) -> dict[str, str | None]:
    out = {}
    tok = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    for g in ids:
        req = urllib.request.Request(f"https://api.github.com/advisories/{g}",
                                     headers={"Accept": "application/vnd.github+json",
                                              **({"Authorization": f"Bearer {tok}"} if tok else {})})
        with urllib.request.urlopen(req, timeout=30) as r:
            d = json.load(r)
        out[g] = next((v.get("first_patched_version") for v in d.get("vulnerabilities") or []
                       if v.get("first_patched_version")), None)
    return out


def check(audit: dict, allow: object, today: dt.date, patched: dict[str, str | None]) -> list[str]:
    found = advisories(audit)
    es = entries(allow)
    listed = {(e["advisory"], e["package"]): e for e in es}
    bad = []
    for g, p in sorted(found - set(listed)):
        bad.append(f"unlisted advisory: {g} in {p}")
    for (g, p), e in sorted(listed.items()):
        if (g, p) not in found:
            bad.append(f"stale entry: {g} in {p} matches no advisory in the audit")
        a, x = dt.date.fromisoformat(e["added"]), dt.date.fromisoformat(e["expires"])
        if (x - a).days > MAX_DAYS:
            bad.append(f"entry {g} in {p}: expiry {e['expires']} is more than {MAX_DAYS} days after {e['added']}")
        if today > x:
            bad.append(f"expired entry: {g} in {p} (expired {e['expires']})")
        if g not in patched:
            raise Bad(f"advisory database: no answer for {g}")
        if patched[g]:
            bad.append(f"listed advisory now patched: {g} in {p} (first patched {patched[g]}): update, don't list")
    return bad


def main(argv: list[str]) -> int:
    args, today, patched_file = [], dt.date.today(), None
    it = iter(argv)
    for a in it:
        if a == "--today":
            today = dt.date.fromisoformat(next(it))
        elif a == "--patched":
            patched_file = next(it)
        else:
            args.append(a)
    if len(args) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        audit = json.load(open(args[0], encoding="utf-8"))
    except (OSError, ValueError) as e:
        print(f"frontend audit: cannot read the audit: {e}", file=sys.stderr)
        return 2
    n = compared(audit)
    if n <= 0:
        print("frontend audit: the audit compared 0 packages", file=sys.stderr)
        return 2
    try:
        allow = json.load(open(args[1], encoding="utf-8"))
    except (OSError, ValueError) as e:
        print(f"frontend audit: malformed allow list: {e}")
        return 1
    try:
        ids = sorted({e["advisory"] for e in entries(allow)})
        if patched_file:
            patched = json.load(open(patched_file, encoding="utf-8"))
        else:
            try:
                patched = first_patched(ids)
            except OSError as e:
                print(f"frontend audit: cannot ask the advisory database: {e}", file=sys.stderr)
                return 2
        bad = check(audit, allow, today, patched)
    except Bad as e:
        print(f"frontend audit: malformed: {e}")
        return 1
    for b in bad:
        print(f"frontend audit: {b}")
    print(f"frontend audit: {n} packages, {len(advisories(audit))} advisories, "
          f"{len(entries(allow))} allow-list entries, {len(bad)} findings")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.exit(main(sys.argv[1:]))
