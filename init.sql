create table if not exists quotes
(
    id         serial primary key        not null,
    content    text                      not null,
    author_id  text                      not null,
    created_at timestamptz default now() not null,
    sent_at    timestamptz               not null
);

create table if not exists votes
(
    quote_id   int                       not null,
    vote       int                       not null,
    created_at timestamptz default now() not null
);