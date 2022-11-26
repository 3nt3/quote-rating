"""
THIS SCRIPT FIXES MY STUPIDITY

I changed the db scheme and it was dumb in the beginning so I kinda lost everything because I was too lazy to fix my code

It is *EXTREMELY* slow. It took like 5 minutes for 1000 entries (because it's very inefficient)
"""


import psycopg2

conn_old = psycopg2.connect("dbname=quotes user=quotes password=kpI2Pq5TZZ4z5VadGul2H85gqs0fBdMj host=localhost port=5433")
conn_new = psycopg2.connect("dbname=quotes user=quotes password=kpI2Pq5TZZ4z5VadGul2H85gqs0fBdMj host=192.168.69.1 port=5435")

cur_old = conn_old.cursor()
cur_new = conn_new.cursor()

cur_old.execute("select * from votes")
old_votes = cur_old.fetchall()

for old_vote in old_votes:
    (msg_id, vote, old_vote_created_at) = old_vote
    # print(msg_id, vote)

    cur_old.execute("select * from quotes where id = %s", [msg_id])
    old_quote = cur_old.fetchone()
    if old_quote is None:
        continue

    old_sent_at = old_quote[4]
    old_author_id = old_quote[2]

    cur_new.execute("select * from quotes where sent_at = %s and author_id = %s", [old_sent_at, old_author_id])
    new_quote = cur_new.fetchone()
    if new_quote is None:
        continue

    new_id = new_quote[0]


    # print(old_sent_at, old_author_id)

    cur_new.execute('insert into votes (quote_id, vote, created_at) values (%s, %s, %s)', [new_id, vote, old_vote_created_at])
    conn_new.commit()

