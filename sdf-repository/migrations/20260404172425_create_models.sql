CREATE TABLE models (
    id SERIAL PRIMARY KEY,
    model JSONB NOT NULL,
    version integer[3] NOT NULL,
    namespace text NOT NULL,
    lineage text,
    UNIQUE NULLS NOT DISTINCT (lineage, namespace, version)
);
