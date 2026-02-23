-- Text Compression domain tables
create table if not exists public.compression_jobs (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    algorithm text not null default 'lz4' check (algorithm in ('lz4', 'zstd', 'brotli', 'gzip', 'deflate', 'snappy')),
    original_size_bytes bigint not null,
    compressed_size_bytes bigint,
    compression_ratio double precision,
    level integer default 3,
    status text default 'pending',
    input_encoding text default 'utf-8',
    created_at timestamptz default now(),
    completed_at timestamptz
);
create table if not exists public.dictionaries (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    name text not null,
    algorithm text not null check (algorithm in ('zstd', 'brotli')),
    training_samples integer default 0,
    dict_size_bytes bigint,
    avg_ratio_improvement double precision,
    dict_data bytea,
    created_at timestamptz default now()
);
create index idx_compression_jobs_user on public.compression_jobs(user_id);
create index idx_compression_jobs_algo on public.compression_jobs(algorithm);
create index idx_dictionaries_user on public.dictionaries(user_id);
