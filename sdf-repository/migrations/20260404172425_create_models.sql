CREATE TABLE models (
    id SERIAL PRIMARY KEY,
    model JSONB NOT NULL,
    version text NOT NULL,
    namespace text NOT NULL,
    lineage text,
    UNIQUE NULLS NOT DISTINCT (lineage, namespace, version)
);
