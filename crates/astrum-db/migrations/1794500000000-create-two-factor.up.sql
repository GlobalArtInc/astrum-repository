CREATE TABLE user_second_factor (
    user_id text PRIMARY KEY REFERENCES security_user (id) ON UPDATE CASCADE ON DELETE CASCADE,
    requirement text NOT NULL DEFAULT 'policy' CHECK (requirement IN ('policy', 'required', 'exempt')),
    enroll_by timestamptz,
    totp_secret text,
    totp_confirmed_at timestamptz,
    totp_last_step bigint NOT NULL DEFAULT 0,
    webauthn_handle uuid NOT NULL DEFAULT gen_random_uuid(),
    failed_codes integer NOT NULL DEFAULT 0,
    failed_since timestamptz,
    locked_until timestamptz,
    lock_level integer NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE user_passkey (
    id uuid PRIMARY KEY,
    user_id text NOT NULL REFERENCES security_user (id) ON UPDATE CASCADE ON DELETE CASCADE,
    name text NOT NULL,
    credential_id bytea NOT NULL UNIQUE,
    rp_id text NOT NULL,
    passkey jsonb NOT NULL,
    backup_eligible boolean NOT NULL,
    backed_up boolean NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);

CREATE INDEX user_passkey_by_user ON user_passkey (user_id);

CREATE TABLE user_recovery_code (
    user_id text NOT NULL REFERENCES security_user (id) ON UPDATE CASCADE ON DELETE CASCADE,
    code_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    used_at timestamptz,
    PRIMARY KEY (user_id, code_hash)
);

CREATE TABLE second_factor_ceremony (
    handle_hash text PRIMARY KEY,
    kind text NOT NULL CHECK (kind IN ('sign-in', 'totp-enroll', 'passkey-enroll')),
    user_id text NOT NULL REFERENCES security_user (id) ON UPDATE CASCADE ON DELETE CASCADE,
    realm text NOT NULL,
    state jsonb,
    attempts integer NOT NULL DEFAULT 0,
    client_ip text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL
);

CREATE INDEX second_factor_ceremony_expiry ON second_factor_ceremony (expires_at);

ALTER TABLE user_session ADD COLUMN restricted boolean NOT NULL DEFAULT false;
