CREATE TABLE content_policy (
    name text PRIMARY KEY,
    notes text NOT NULL DEFAULT '',
    vulnerabilities jsonb NOT NULL DEFAULT '{"mode": "off"}',
    licenses jsonb NOT NULL DEFAULT '{"mode": "off"}',
    signatures jsonb NOT NULL DEFAULT '{"mode": "off"}',
    revision bigint NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX content_policy_name_lower ON content_policy (lower(name));

CREATE TABLE policy_exception (
    id uuid PRIMARY KEY,
    policy text REFERENCES content_policy (name) ON UPDATE CASCADE ON DELETE CASCADE,
    rule text NOT NULL CHECK (rule IN ('vulnerability', 'license', 'signature')),
    advisory text,
    license text,
    format text,
    namespace text,
    name text,
    version text,
    digest text,
    reason text NOT NULL,
    expires_at timestamptz,
    created_by text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (advisory IS NOT NULL OR license IS NOT NULL OR name IS NOT NULL OR digest IS NOT NULL)
);

CREATE INDEX policy_exception_rule ON policy_exception (rule, policy);

CREATE INDEX policy_exception_expiry ON policy_exception (expires_at) WHERE expires_at IS NOT NULL;

CREATE TABLE trusted_key (
    name text PRIMARY KEY,
    algorithm text NOT NULL,
    public_key text NOT NULL,
    fingerprint text NOT NULL UNIQUE,
    notes text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX trusted_key_name_lower ON trusted_key (lower(name));

CREATE TABLE advisory (
    id text PRIMARY KEY,
    aliases text[] NOT NULL DEFAULT '{}',
    summary text NOT NULL DEFAULT '',
    severity text NOT NULL,
    score numeric(3, 1),
    vector text,
    malicious boolean NOT NULL DEFAULT false,
    published_at timestamptz,
    modified_at timestamptz NOT NULL,
    withdrawn_at timestamptz,
    document jsonb NOT NULL
);

CREATE INDEX advisory_aliases ON advisory USING gin (aliases);

CREATE TABLE advisory_affected (
    advisory_id text NOT NULL REFERENCES advisory (id) ON DELETE CASCADE,
    ecosystem text NOT NULL,
    name text NOT NULL,
    ranges jsonb NOT NULL DEFAULT '[]',
    versions text[] NOT NULL DEFAULT '{}',
    PRIMARY KEY (advisory_id, ecosystem, name)
);

CREATE INDEX advisory_affected_package ON advisory_affected (ecosystem, name);

CREATE TABLE advisory_feed (
    ecosystem text PRIMARY KEY,
    etag text,
    records integer NOT NULL DEFAULT 0,
    synced_at timestamptz,
    attempted_at timestamptz,
    last_error text
);

CREATE TABLE sca_package (
    id bigserial PRIMARY KEY,
    ecosystem text NOT NULL,
    name text NOT NULL,
    version text NOT NULL,
    purl text NOT NULL,
    matched_at timestamptz,
    UNIQUE (ecosystem, name, version)
);

CREATE INDEX sca_package_unmatched ON sca_package (id) WHERE matched_at IS NULL;

CREATE TABLE sca_match (
    package_id bigint NOT NULL REFERENCES sca_package (id) ON DELETE CASCADE,
    advisory_id text NOT NULL REFERENCES advisory (id) ON DELETE CASCADE,
    fixed text,
    first_seen_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (package_id, advisory_id)
);

CREATE INDEX sca_match_advisory ON sca_match (advisory_id);

CREATE TABLE sca_inventory (
    id bigserial PRIMARY KEY,
    key text NOT NULL UNIQUE,
    source text NOT NULL,
    distro text,
    extractor integer NOT NULL,
    extracted_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE sca_inventory_package (
    inventory_id bigint NOT NULL REFERENCES sca_inventory (id) ON DELETE CASCADE,
    package_id bigint NOT NULL REFERENCES sca_package (id) ON DELETE CASCADE,
    location text NOT NULL DEFAULT '',
    PRIMARY KEY (inventory_id, package_id)
);

CREATE INDEX sca_inventory_package_package ON sca_inventory_package (package_id);

CREATE TABLE oci_layer_facts (
    digest text PRIMARY KEY,
    extractor integer NOT NULL,
    facts jsonb NOT NULL,
    extracted_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE component_scan (
    component_id bigint PRIMARY KEY REFERENCES component (id) ON DELETE CASCADE,
    repository text NOT NULL,
    inventory_id bigint REFERENCES sca_inventory (id) ON DELETE SET NULL,
    digest text,
    state text NOT NULL,
    error text,
    scanned_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX component_scan_inventory ON component_scan (inventory_id);

CREATE INDEX component_scan_repository ON component_scan (repository, state);

CREATE TABLE scan_queue (
    component_id bigint PRIMARY KEY REFERENCES component (id) ON DELETE CASCADE,
    reason text NOT NULL,
    attempts integer NOT NULL DEFAULT 0,
    queued_at timestamptz NOT NULL DEFAULT now(),
    next_attempt_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX scan_queue_due ON scan_queue (next_attempt_at);

CREATE TABLE component_license (
    component_id bigint PRIMARY KEY REFERENCES component (id) ON DELETE CASCADE,
    declared text NOT NULL,
    source text NOT NULL,
    expression text,
    ids text[] NOT NULL DEFAULT '{}',
    state text NOT NULL
);

CREATE INDEX component_license_ids ON component_license USING gin (ids);

CREATE TABLE policy_verdict (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    subject text NOT NULL,
    policy text NOT NULL,
    component_id bigint REFERENCES component (id) ON DELETE CASCADE,
    blocked boolean NOT NULL,
    state text NOT NULL,
    critical integer NOT NULL DEFAULT 0,
    high integer NOT NULL DEFAULT 0,
    medium integer NOT NULL DEFAULT 0,
    low integer NOT NULL DEFAULT 0,
    unknown integer NOT NULL DEFAULT 0,
    max_severity text,
    license_state text,
    reasons jsonb NOT NULL DEFAULT '[]',
    policy_revision bigint NOT NULL,
    computed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository, subject)
);

CREATE INDEX policy_verdict_component ON policy_verdict (component_id);

CREATE INDEX policy_verdict_findings ON policy_verdict (repository, max_severity) WHERE max_severity IS NOT NULL;

CREATE TABLE policy_dirty (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    subject text NOT NULL,
    queued_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository, subject)
);

CREATE INDEX policy_dirty_queued_at ON policy_dirty (queued_at);

CREATE TABLE docker_manifest_child (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    parent text NOT NULL,
    child text NOT NULL,
    PRIMARY KEY (repository, parent, child)
);

CREATE INDEX docker_manifest_child_child ON docker_manifest_child (repository, child);

CREATE TABLE oci_signature_check (
    repository text NOT NULL REFERENCES repository (name) ON UPDATE CASCADE ON DELETE CASCADE,
    digest text NOT NULL,
    fingerprint text NOT NULL,
    keys_revision bigint NOT NULL,
    state text NOT NULL,
    signer text,
    detail jsonb NOT NULL DEFAULT '{}',
    checked_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository, digest)
);

ALTER TABLE notification_change ADD COLUMN detail jsonb;
