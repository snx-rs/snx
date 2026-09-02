# snx

snx is an experimental, opiniated and batteries-included web framework that
tries to be a breath of fresh air by making different choices than other popular
web frameworks and (hopefully) by making these choices, can give you a better
experience developing and maintaining efficient and robust web applications.

check out the documentation at [`./docs/1-introduction.md`](./docs/1-introduction.md) to get started.

## llms and generative ai

all code and documentation is written, reasoned and thought about by a real human
being.

## core values and goals

###### code is an art

reading code while working on web applications using snx should be clear and
simple. a new snx project should feel like an empty canvas and writing code
should feel like painting and should be joyful.

###### embrace your stack

your choice of database, cache, blob storage etc. are not abstracted away and
are part of your application. snx goes deep on what you choose and gives handles
for it, think [Postgres RLS](https://www.postgresql.org/docs/current/ddl-rowsecurity.html), [Redis Streams](https://redis.io/docs/latest/develop/data-types/streams/) and [S3 Presigning](https://docs.aws.amazon.com/AmazonS3/latest/userguide/ShareObjectPreSignedURL.html).

###### if it compiles, it works

no orm, just the best slice of one. you get the good parts (free crud, generated
types, zero boilerplate) derived from your schema, and none of the rest: no
query DSL, no lazy loading, no hidden queries. everything beyond crud is plain
sql. models and queries are type-checked against your schema at compile-time.

###### strict at every boundary

snx inverts Postel's law. unknown/malformed input is an error, also every
possible response must be defined beforehand, both successful ones and errors.

## history and roadmap

| version | description                                                               | status |
| ------- | ------------------------------------------------------------------------- | ------ |
| 0.1.0   | everything required to create a good JSON api                             |        |
| 0.0.x   | ...                                                                       |        |
| 0.0.6   | full rewrite                                                              | busy   |
| 0.0.5   | json io and experimenting with database interaction                       | done   |
| 0.0.4   | improved http handling and routing                                        | done   |
| 0.0.3   | middleware, basic tracing and improved routing                            | done   |
| 0.0.2   | first implementation of simple router and handlers                        | done   |
