ALTER TABLE content_policy ADD COLUMN cooldown jsonb NOT NULL DEFAULT '{"mode": "off"}';

ALTER TABLE asset ADD COLUMN upstream_published_at timestamptz;

ALTER TABLE policy_exception DROP CONSTRAINT policy_exception_rule_check,
    ADD CONSTRAINT policy_exception_rule_check
    CHECK (rule IN ('vulnerability', 'license', 'signature', 'cooldown'));

CREATE TABLE cooldown_wait (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    component_id bigint NOT NULL REFERENCES component (id) ON DELETE CASCADE,
    published_at timestamptz NOT NULL,
    available_at timestamptz NOT NULL,
    blocked boolean NOT NULL,
    refusals integer NOT NULL DEFAULT 1,
    first_refused_at timestamptz NOT NULL DEFAULT now(),
    last_refused_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository, component_id)
);

CREATE INDEX cooldown_wait_due ON cooldown_wait (available_at);
