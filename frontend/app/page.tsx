import Link from 'next/link';

const features = [
  { title: 'Exception-Based Compression', desc: 'Predictive model with exception encoding for superior text ratios' },
  { title: 'Multi-Algorithm Engine', desc: 'Hybrid of predictive coding, entropy encoding, and exception-based methods' },
  { title: 'Batch Processing', desc: 'Compress thousands of documents in a single API call' },
  { title: 'Text Analysis', desc: 'Shannon entropy, language detection, and compression estimations' },
];

const algorithms = ['Exception-Based', 'Predictive Coding', 'Entropy Encoding', 'Hybrid (5.1x)'];

export default function Home() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b border-border">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <h1 className="text-xl font-bold">ALICE Text Compression</h1>
          <div className="flex gap-3">
            <Link href="/auth/login" className="px-4 py-2 text-sm text-muted-foreground hover:text-foreground">Sign in</Link>
            <Link href="/auth/register" className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-md hover:opacity-90">Get Started</Link>
          </div>
        </div>
      </header>
      <main>
        <section className="max-w-6xl mx-auto px-6 py-20 text-center">
          <h2 className="text-4xl font-bold mb-4">Exception-Based Text Compression</h2>
          <p className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto">Compress text data at up to 5.1x ratio using predictive coding + entropy encoding. Built with Rust SIMD for maximum throughput.</p>
          <Link href="/dashboard/console" className="px-6 py-3 bg-primary text-primary-foreground rounded-md font-medium hover:opacity-90">Launch Console</Link>
        </section>
        <section className="max-w-6xl mx-auto px-6 pb-20">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {features.map((f) => (
              <div key={f.title} className="border border-border rounded-lg p-6">
                <h3 className="font-semibold mb-2">{f.title}</h3>
                <p className="text-sm text-muted-foreground">{f.desc}</p>
              </div>
            ))}
          </div>
        </section>
        <section className="max-w-6xl mx-auto px-6 pb-20">
          <h3 className="text-xl font-semibold mb-4 text-center">Supported Algorithms</h3>
          <div className="flex flex-wrap justify-center gap-3">
            {algorithms.map((a) => (
              <span key={a} className="px-4 py-2 bg-muted rounded-full text-sm">{a}</span>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}
