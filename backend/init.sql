create table quotes
(
    id         serial
        primary key,
    content    text                                   not null,
    author_id  text                                   not null,
    created_at timestamp with time zone default now() not null,
    sent_at    timestamp with time zone               not null,
    avatar_url text
);

alter table quotes
    owner to quotes;

create table votes
(
    quote_id   integer                                not null,
    vote       integer                                not null,
    created_at timestamp with time zone default now() not null
);

alter table votes
    owner to quotes;

create table username_cache
(
    user_id  varchar(50)  not null
        constraint username_cache_pk
            primary key,
    username varchar(100) not null
);

alter table username_cache
    owner to quotes;


