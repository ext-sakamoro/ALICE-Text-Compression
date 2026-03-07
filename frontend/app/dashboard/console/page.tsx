'use client';
import { useState } from 'react';
import { useTextStore } from '@/lib/hooks/use-store';

const API = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const ALGORITHMS = ['hybrid', 'exception', 'predictive', 'entropy'];

type Tab = 'compress' | 'analyze' | 'batch' | 'algorithms';

export default function ConsolePage() {
  const [tab, setTab] = useState<Tab>('compress');
  const { inputText, setInputText, algorithm, setAlgorithm, result, setResult, loading, setLoading } = useTextStore();
  const [batchTexts, setBatchTexts] = useState('');

  const doFetch = async (path: string, body: unknown) => {
    setLoading(true);
    setResult(null);
    try {
      const r = await fetch(`${API}${path}`, { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-API-Key': 'demo' }, body: JSON.stringify(body) });
      const data = await r.json();
      setResult(data);
    } catch (e) {
      setResult({ error: (e as Error).message });
    } finally {
      setLoading(false);
    }
  };

  const doGet = async (path: string) => {
    setLoading(true);
    setResult(null);
    try {
      const r = await fetch(`${API}${path}`, { headers: { 'X-API-Key': 'demo' } });
      const data = await r.json();
      setResult(data);
    } catch (e) {
      setResult({ error: (e as Error).message });
    } finally {
      setLoading(false);
    }
  };

  const tabs: { key: Tab; label: string }[] = [
    { key: 'compress', label: 'Compress' },
    { key: 'analyze', label: 'Analyze' },
    { key: 'batch', label: 'Batch' },
    { key: 'algorithms', label: 'Algorithms' },
  ];

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Text Compression Console</h1>

      <div className="flex gap-1 border-b border-border">
        {tabs.map((t) => (
          <button key={t.key} onClick={() => { setTab(t.key); setResult(null); }}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${tab === t.key ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'}`}>
            {t.label}
          </button>
        ))}
      </div>

      {/* Compress Tab */}
      {tab === 'compress' && (
        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium block mb-1">Algorithm</label>
            <div className="flex gap-2">
              {ALGORITHMS.map((a) => (
                <button key={a} onClick={() => setAlgorithm(a)}
                  className={`px-3 py-1.5 rounded-md text-xs font-medium ${algorithm === a ? 'bg-emerald-600 text-white' : 'bg-muted text-muted-foreground hover:bg-accent'}`}>
                  {a}
                </button>
              ))}
            </div>
          </div>
          <div>
            <label className="text-sm font-medium block mb-1">Input Text</label>
            <textarea rows={8} value={inputText} onChange={(e) => setInputText(e.target.value)}
              placeholder="Paste text to compress..."
              className="w-full px-3 py-2 border border-input rounded-md bg-background text-sm font-mono resize-none" />
          </div>
          <div className="flex gap-2">
            <button onClick={() => doFetch('/api/v1/text/compress', { text: inputText, algorithm })}
              disabled={loading || !inputText.trim()}
              className="px-4 py-2 bg-emerald-600 text-white rounded-md text-sm font-medium hover:bg-emerald-700 disabled:opacity-50">
              {loading ? 'Compressing...' : 'Compress'}
            </button>
            <button onClick={() => doFetch('/api/v1/text/decompress', { data: inputText, algorithm })}
              disabled={loading || !inputText.trim()}
              className="px-4 py-2 bg-muted text-muted-foreground rounded-md text-sm font-medium hover:bg-accent disabled:opacity-50">
              Decompress
            </button>
          </div>
          {result && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              <h3 className="text-sm font-semibold">Result</h3>
              {'error' in result ? (
                <p className="text-sm text-red-500">{String(result.error)}</p>
              ) : (
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                  {result.algorithm != null && <Stat label="Algorithm" value={String(result.algorithm)} />}
                  {result.original_size != null && <Stat label="Original" value={`${result.original_size} bytes`} />}
                  {result.compressed_size != null && <Stat label="Compressed" value={`${result.compressed_size} bytes`} />}
                  {result.ratio != null && <Stat label="Ratio" value={`${Number(result.ratio).toFixed(2)}x`} accent />}
                  {result.elapsed_us != null && <Stat label="Time" value={`${result.elapsed_us} us`} />}
                  {result.job_id != null && <Stat label="Job ID" value={String(result.job_id).slice(0, 8)} />}
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Analyze Tab */}
      {tab === 'analyze' && (
        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium block mb-1">Text to Analyze</label>
            <textarea rows={8} value={inputText} onChange={(e) => setInputText(e.target.value)}
              placeholder="Paste text to analyze..."
              className="w-full px-3 py-2 border border-input rounded-md bg-background text-sm font-mono resize-none" />
          </div>
          <button onClick={() => doFetch('/api/v1/text/analyze', { text: inputText })}
            disabled={loading || !inputText.trim()}
            className="px-4 py-2 bg-emerald-600 text-white rounded-md text-sm font-medium hover:bg-emerald-700 disabled:opacity-50">
            {loading ? 'Analyzing...' : 'Analyze'}
          </button>
          {result && !('error' in result) && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              <h3 className="text-sm font-semibold">Analysis</h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <Stat label="Characters" value={String(result.char_count ?? '-')} />
                <Stat label="Bytes" value={String(result.byte_count ?? '-')} />
                <Stat label="Words" value={String(result.word_count ?? '-')} />
                <Stat label="Lines" value={String(result.line_count ?? '-')} />
                <Stat label="Unique Chars" value={String(result.unique_chars ?? '-')} />
                <Stat label="Entropy" value={`${Number(result.entropy ?? 0).toFixed(3)} bits`} accent />
                <Stat label="Language" value={String(result.language_hint ?? '-')} />
              </div>
              {result.estimated_ratios != null && (
                <div className="mt-3">
                  <h4 className="text-xs font-semibold text-muted-foreground mb-2">Estimated Compression Ratios</h4>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                    {Object.entries(result.estimated_ratios as Record<string, number>).map(([k, v]) => (
                      <div key={k} className="px-3 py-2 bg-muted rounded-md">
                        <div className="text-xs text-muted-foreground">{k}</div>
                        <div className="text-sm font-semibold text-emerald-500">{Number(v).toFixed(2)}x</div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Batch Tab */}
      {tab === 'batch' && (
        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium block mb-1">Texts (one per line)</label>
            <textarea rows={10} value={batchTexts} onChange={(e) => setBatchTexts(e.target.value)}
              placeholder="Enter multiple texts, one per line..."
              className="w-full px-3 py-2 border border-input rounded-md bg-background text-sm font-mono resize-none" />
          </div>
          <div className="flex gap-2 items-center">
            <select value={algorithm} onChange={(e) => setAlgorithm(e.target.value)}
              className="px-3 py-2 border border-input rounded-md bg-background text-sm">
              {ALGORITHMS.map((a) => <option key={a} value={a}>{a}</option>)}
            </select>
            <button onClick={() => doFetch('/api/v1/text/batch', { texts: batchTexts.split('\n').filter(Boolean), algorithm })}
              disabled={loading || !batchTexts.trim()}
              className="px-4 py-2 bg-emerald-600 text-white rounded-md text-sm font-medium hover:bg-emerald-700 disabled:opacity-50">
              {loading ? 'Processing...' : 'Batch Compress'}
            </button>
          </div>
          {result && !('error' in result) && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              <h3 className="text-sm font-semibold">Batch Results ({String(result.total_items ?? 0)} items)</h3>
              {Array.isArray(result.results) && (
                <table className="w-full text-sm">
                  <thead><tr className="text-left text-muted-foreground border-b border-border">
                    <th className="py-1 pr-4">#</th><th className="py-1 pr-4">Original</th><th className="py-1 pr-4">Compressed</th><th className="py-1">Ratio</th>
                  </tr></thead>
                  <tbody>
                    {(result.results as Array<Record<string, unknown>>).map((r, i) => (
                      <tr key={i} className="border-b border-border/50">
                        <td className="py-1 pr-4 text-muted-foreground">{Number(r.index ?? i)}</td>
                        <td className="py-1 pr-4">{String(r.original_size)} B</td>
                        <td className="py-1 pr-4">{String(r.compressed_size)} B</td>
                        <td className="py-1 text-emerald-500 font-medium">{Number(r.ratio).toFixed(2)}x</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          )}
        </div>
      )}

      {/* Algorithms Tab */}
      {tab === 'algorithms' && (
        <div className="space-y-4">
          <button onClick={() => doGet('/api/v1/text/algorithms')}
            disabled={loading}
            className="px-4 py-2 bg-emerald-600 text-white rounded-md text-sm font-medium hover:bg-emerald-700 disabled:opacity-50">
            {loading ? 'Loading...' : 'Load Algorithms'}
          </button>
          {result && Array.isArray(result) && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {(result as Array<Record<string, unknown>>).map((a) => (
                <div key={String(a.name)} className="border border-border rounded-lg p-4">
                  <h3 className="font-semibold">{String(a.name)}</h3>
                  <p className="text-sm text-muted-foreground mt-1">{String(a.description)}</p>
                  <div className="flex gap-4 mt-2 text-xs">
                    <span className="text-emerald-500 font-medium">Ratio: {String(a.typical_ratio)}x</span>
                    <span className="text-muted-foreground">Best for: {String(a.best_for)}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function Stat({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="px-3 py-2 bg-muted rounded-md">
      <div className="text-xs text-muted-foreground">{label}</div>
      <div className={`text-sm font-semibold ${accent ? 'text-emerald-500' : ''}`}>{value}</div>
    </div>
  );
}
