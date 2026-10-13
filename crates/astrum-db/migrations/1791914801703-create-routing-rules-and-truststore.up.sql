CREATE TABLE routing_rule (
    name text PRIMARY KEY,
    description text NOT NULL DEFAULT '',
    mode text NOT NULL,
    matchers jsonb NOT NULL DEFAULT '[]',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX routing_rule_name_lower ON routing_rule (lower(name));

CREATE TABLE ssl_certificate (
    id text PRIMARY KEY,
    pem text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
