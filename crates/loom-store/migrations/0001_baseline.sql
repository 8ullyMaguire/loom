create table if not exists feeds (
  uri text primary key,
  did text not null,
  rkey text not null,
  display_name text not null,
  description text not null default '',
  publisher text not null default '',
  source text not null check (source in ('leftist','bnwo','atproto')),
  topic text[] not null default '{}',
  lang text[] not null default '{}',
  ranking_mode text not null default '',
  max_posts integer,
  likes_count integer,
  first_seen_at timestamptz not null default now(),
  last_seen_at timestamptz not null default now(),
  alive boolean not null default true
);
create index if not exists feeds_source_idx on feeds(source);
create index if not exists feeds_publisher_idx on feeds(publisher);

create table if not exists bundles (
  id uuid primary key,
  owner_key text not null,
  name text not null check (length(name) between 1 and 80),
  description text not null default '',
  edit_token_hash bytea not null,
  parent_id uuid references bundles(id),
  version integer not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  deleted_at timestamptz
);
create index if not exists bundles_owner_idx on bundles(owner_key);

create table if not exists bundle_feeds (
  bundle_id uuid not null references bundles(id) on delete cascade,
  feed_uri text not null references feeds(uri),
  position integer not null check (position >= 0),
  primary key (bundle_id, feed_uri),
  unique (bundle_id, position)
);

create table if not exists tag_votes (
  contributor_key text not null,
  feed_uri text not null references feeds(uri),
  tag text not null,
  confidence integer not null check (confidence between 0 and 100),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (contributor_key, feed_uri, tag)
);
create index if not exists tag_votes_feed_tag_idx on tag_votes(feed_uri, tag);

create table if not exists bundle_links (
  bundle_id uuid not null references bundles(id) on delete cascade,
  visitor_key text not null,
  occurred_at timestamptz not null default now(),
  primary key (bundle_id, visitor_key, occurred_at)
);

create table if not exists reports (
  id bigserial primary key,
  kind text not null check (kind in ('bundle','tag')),
  target_id text not null,
  reason text not null,
  reporter_key text not null,
  created_at timestamptz not null default now()
);
create index if not exists reports_created_idx on reports(created_at);

create table if not exists reader_signals (
  visitor_key text not null,
  feed_uri text not null references feeds(uri),
  signal text not null check (signal in ('open','save','hide')),
  weight real not null,
  occurred_at timestamptz not null default now(),
  primary key (visitor_key, feed_uri, signal, occurred_at)
);

create table if not exists idempotency_keys (
  key text primary key,
  route text not null,
  response jsonb not null,
  created_at timestamptz not null default now()
);

create table if not exists events (
  id bigserial primary key,
  kind text not null,
  payload jsonb not null,
  occurred_at timestamptz not null default now()
);
