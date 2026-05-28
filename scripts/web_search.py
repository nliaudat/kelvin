#!/usr/bin/env python3
"""Web search for similar projects, references, and patents for Kelvin cryptosystem."""

import urllib.request
import urllib.parse
import json
import sys
import re
import time

def fetch(url, timeout=20):
    """Fetch a URL with a user-agent header."""
    req = urllib.request.Request(
        url,
        headers={
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
    )
    resp = urllib.request.urlopen(req, timeout=timeout)
    return resp.read().decode('utf-8', errors='replace')

def search_arxiv(query, max_results=10):
    """Search arXiv API."""
    params = {
        'search_query': query,
        'start': 0,
        'max_results': max_results,
        'sortBy': 'submittedDate',
        'sortOrder': 'descending'
    }
    url = f"http://export.arxiv.org/api/query?{urllib.parse.urlencode(params)}"
    print(f"\n=== arXiv Search: {query[:80]}... ===")
    try:
        xml = fetch(url)
        # Simple parsing for entries
        entries = re.findall(r'<entry>(.*?)</entry>', xml, re.DOTALL)
        if not entries:
            print("  No results found.")
            return
        for entry in entries:
            title = re.search(r'<title>(.*?)</title>', entry, re.DOTALL)
            published = re.search(r'<published>(.*?)</published>', entry)
            link = re.search(r'<id>(.*?)</id>', entry)
            summary = re.search(r'<summary>(.*?)</summary>', entry, re.DOTALL)
            if title:
                t = title.group(1).strip()
                print(f"  Title: {t[:120]}")
            if published:
                print(f"  Published: {published.group(1).strip()}")
            if link:
                print(f"  Link: {link.group(1).strip()}")
            if summary:
                s = summary.group(1).strip()[:200]
                print(f"  Summary: {s}...")
            print()
    except Exception as e:
        print(f"  Error: {e}")

def search_google_patents(query):
    """Search Google Patents."""
    encoded = urllib.parse.quote(query)
    url = f"https://patents.google.com/?q={encoded}&language=ENGLISH&num=20"
    print(f"\n=== Google Patents Search: {query[:80]}... ===")
    try:
        html = fetch(url)
        # Look for patent titles and numbers
        # Google Patents uses structured data
        titles = re.findall(r'<meta itemprop="name" content="([^"]+)"', html)
        patents = re.findall(r'(US\d{5,12}(?:B2|A1|A|B1)?)', html)
        if titles:
            for t in titles[:10]:
                print(f"  Title: {t[:150]}")
        if patents:
            for p in patents[:15]:
                print(f"  Patent: {p}")
        if not titles and not patents:
            # Try alternative patterns
            results = re.findall(r'class="result-title"[^>]*>([^<]+)<', html)
            if results:
                for r in results[:10]:
                    print(f"  Result: {r.strip()[:150]}")
            else:
                print("  No results found (or page format unrecognized).")
    except Exception as e:
        print(f"  Error: {e}")

def search_github(query):
    """Search GitHub repositories."""
    encoded = urllib.parse.quote(query)
    url = f"https://api.github.com/search/repositories?q={encoded}&sort=stars&order=desc&per_page=10"
    print(f"\n=== GitHub Search: {query[:80]}... ===")
    try:
        req = urllib.request.Request(
            url,
            headers={'User-Agent': 'KelvinSearch/1.0', 'Accept': 'application/vnd.github.v3+json'}
        )
        resp = urllib.request.urlopen(req, timeout=15)
        data = json.loads(resp.read().decode('utf-8'))
        items = data.get('items', [])
        if not items:
            print("  No results found.")
            return
        for item in items[:10]:
            print(f"  Repo: {item['full_name']}")
            print(f"  Stars: {item['stargazers_count']}")
            print(f"  Description: {item.get('description', 'N/A')[:150]}")
            print(f"  URL: {item['html_url']}")
            print()
    except Exception as e:
        print(f"  Error: {e}")

def search_iacr(query):
    """Search IACR ePrint."""
    encoded = urllib.parse.quote(query)
    url = f"https://eprint.iacr.org/search?query={encoded}"
    print(f"\n=== IACR ePrint Search: {query[:80]}... ===")
    try:
        html = fetch(url)
        # Look for paper entries
        papers = re.findall(r'<a href="/(\d{4}/\d+)"[^>]*>([^<]+)</a>', html)
        if papers:
            for pid, title in papers[:15]:
                print(f"  {pid}: {title.strip()[:150]}")
        else:
            print("  No results found.")
    except Exception as e:
        print(f"  Error: {e}")

def main():
    # === PATENT SEARCHES ===
    print("=" * 70)
    print("PATENT SEARCHES")
    print("=" * 70)
    
    search_google_patents('"n-body" "key derivation" cryptography')
    time.sleep(1)
    search_google_patents('gravitational simulation cryptography key')
    time.sleep(1)
    search_google_patents('orbital mechanics key generation')
    time.sleep(1)
    search_google_patents('chaos based key derivation function')
    time.sleep(1)
    search_google_patents('n body simulation cryptographic key')
    time.sleep(1)
    search_google_patents('gravitational key derivation')
    time.sleep(1)
    search_google_patents('celestial mechanics cryptography')
    time.sleep(1)
    search_google_patents('verlet integration key generation')
    
    # === ACADEMIC SEARCHES ===
    print("\n" + "=" * 70)
    print("ACADEMIC SEARCHES")
    print("=" * 70)
    
    # arXiv searches
    search_arxiv('all:"n body" AND all:simulation AND all:cryptographic AND all:key')
    time.sleep(3)
    search_arxiv('all:"gravitational" AND all:"key derivation" AND all:cryptography')
    time.sleep(3)
    search_arxiv('all:"chaos" AND all:"key derivation" AND all:cryptography')
    time.sleep(3)
    search_arxiv('all:"orbital" AND all:cryptography AND all:key')
    time.sleep(3)
    search_arxiv('all:"n body" AND all:simulation AND all:entropy')
    time.sleep(3)
    search_arxiv('all:"gravitational simulation" AND all:random')
    time.sleep(3)
    search_arxiv('all:"celestial mechanics" AND all:cryptography')
    time.sleep(3)
    search_arxiv('all:"physical unclonable" AND all:simulation')
    time.sleep(3)
    search_arxiv('all:"deterministic chaos" AND all:cryptography')
    time.sleep(3)
    search_arxiv('all:"lyapunov exponent" AND all:cryptography AND all:key')
    
    # IACR ePrint searches
    search_iacr('gravitational OR "n-body" OR orbital')
    time.sleep(1)
    search_iacr('"key derivation" chaos')
    time.sleep(1)
    search_iacr('"physical simulation" cryptography')
    
    # === GITHUB SEARCHES ===
    print("\n" + "=" * 70)
    print("GITHUB SEARCHES")
    print("=" * 70)
    
    search_github('n-body simulation cryptography')
    time.sleep(1)
    search_github('gravitational key derivation')
    time.sleep(1)
    search_github('chaos based cryptography')
    time.sleep(1)
    search_github('orbital cryptography')
    time.sleep(1)
    search_github('physical key derivation')
    time.sleep(1)
    search_github('simulation based encryption')
    
    print("\n" + "=" * 70)
    print("SEARCH COMPLETE")
    print("=" * 70)

if __name__ == '__main__':
    main()
