# Micro-endpoints contract (all frameworks implement this identically)

All responses `Content-Type: application/json`. Errors are out of scope (happy path only).

## GET /json
200 → `{"message":"Hello, World!"}`

## GET /db
200 → `{"id":<int 1..10000>,"randomNumber":<int>}` for one random row of `world`.

## GET /queries?n=K
`n` clamped to [1,500] (missing/invalid → 1). 200 → JSON array of K `{"id","randomNumber"}`,
each from an independent random-id lookup.

## GET /updates?n=K
`n` clamped to [1,500]. For each of K: read a random row, set `randomNumber` to a new random
int, persist. 200 → JSON array of the K updated `{"id","randomNumber"}`.

## Schema
`world(id SERIAL PRIMARY KEY, randomNumber INT NOT NULL)`, seeded 10000 rows,
`randomNumber` initialized to a random int in [1,10000].
