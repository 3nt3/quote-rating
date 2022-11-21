create table if not exists votes
(
    quote_id   integer                                not null,
    vote       integer                                not null,
    created_at timestamp with time zone default now() not null
);

alter table votes
    owner to quotes;

create table if not exists username_cache
(
    user_id  varchar(50)  not null
        constraint username_cache_pk
            primary key,
    username varchar(100) not null
);

alter table username_cache
    owner to quotes;

create table if not exists quotes
(
    id         serial
        constraint quotes_pk
            primary key,
    content    text                                   not null,
    author_id  varchar(50)                            not null,
    sent_at    timestamp with time zone               not null,
    avatar_url text                                   not null,
    message_id text                                   not null,
    channel_id text                                   not null,
    created_at timestamp with time zone default now() not null
);

alter table quotes
    owner to quotes;


