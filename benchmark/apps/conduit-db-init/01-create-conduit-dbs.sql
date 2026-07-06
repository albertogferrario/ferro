-- Shared Postgres server, isolated logical databases for the Conduit benchmark.
-- Both implementations own the same canonical Conduit schema (users, articles, ...),
-- so they cannot share one database. They share the same Postgres *server/container*
-- (same engine, same host resources) with one database each — the standard
-- like-for-like setup. POSTGRES_DB creates `conduit_ferro`; this script adds the rest.
CREATE DATABASE conduit_laravel;
