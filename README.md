# ALICE Text Compression

Exception-based text compression engine — predictive coding, entropy encoding, and hybrid compression via REST API.

**License: AGPL-3.0**

---

## Architecture

```
                    ┌─────────────────┐
                    │   Browser / UI  │
                    │  Next.js :3000  │
                    └────────┬────────┘
                             │ HTTP
                    ┌────────▼────────┐
                    │   API Gateway   │
                    │     :8080       │
                    └────────┬────────┘
                             │ HTTP
                    ┌────────▼────────┐
                    │   Text Engine   │
                    │  Rust/Axum      │
                    │    :8081        │
                    └─────────────────┘
```

| Service | Port | Description |
|---------|------|-------------|
| Frontend | 3000 | Next.js dashboard |
| API Gateway | 8080 | Reverse proxy / auth |
| Text Engine | 8081 | Rust/Axum core engine |

---

## API Endpoints

### POST /api/v1/text/compress

Compress text using the specified algorithm.

**Request:**
```json
{
  "text": "Hello world! Hello world!",
  "algorithm": "hybrid",
  "level": 6
}
```

**Response:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "algorithm": "hybrid",
  "original_size": 25,
  "compressed_size": 5,
  "ratio": 5.1,
  "elapsed_us": 42
}
```

---

### POST /api/v1/text/decompress

Decompress data with the specified algorithm.

**Request:**
```json
{
  "data": "compressed_payload_here",
  "algorithm": "hybrid"
}
```

---

### POST /api/v1/text/analyze

Analyze text characteristics for optimal compression strategy.

**Request:**
```json
{
  "text": "Sample text for analysis"
}
```

**Response:**
```json
{
  "char_count": 24,
  "byte_count": 24,
  "word_count": 4,
  "line_count": 1,
  "unique_chars": 17,
  "entropy": 3.82,
  "language_hint": "latin",
  "estimated_ratios": {
    "exception": 6.7,
    "predictive": 9.4,
    "entropy": 5.9,
    "hybrid": 10.7
  }
}
```

---

### POST /api/v1/text/batch

Batch-compress multiple texts in a single request.

**Request:**
```json
{
  "texts": ["Hello world", "Compression test", "AAAAAAA"],
  "algorithm": "hybrid"
}
```

---

### GET /api/v1/text/algorithms

List available compression algorithms with descriptions and typical ratios.

---

### GET /api/v1/text/stats

Server-wide compression statistics.

---

### GET /health

Health check endpoint.

---

## Algorithms

| Algorithm | Typical Ratio | Best For |
|-----------|--------------|----------|
| exception | 3.2x | Structured text, logs, CSV |
| predictive | 4.5x | Natural language, documents |
| entropy | 2.8x | Mixed content, base64 data |
| hybrid | 5.1x | General purpose (default) |

---

## Quick Start

### Text Engine (Rust)

```bash
cd services/core-engine
cargo build --release
TEXT_ADDR=0.0.0.0:8081 ./target/release/text-engine
```

### Frontend (Next.js)

```bash
cd frontend
npm install
npm run dev
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TEXT_ADDR` | `0.0.0.0:8081` | Text engine bind address |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8080` | API base URL for frontend |

---

## License

AGPL-3.0 — See [LICENSE](LICENSE) for details.
