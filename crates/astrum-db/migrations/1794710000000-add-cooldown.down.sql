DROP INDEX cooldown_wait_due;

DROP TABLE cooldown_wait;

DELETE FROM policy_exception WHERE rule = 'cooldown';

ALTER TABLE policy_exception DROP CONSTRAINT policy_exception_rule_check,
    ADD CONSTRAINT policy_exception_rule_check
    CHECK (rule IN ('vulnerability', 'license', 'signature'));

ALTER TABLE asset DROP COLUMN upstream_published_at;

ALTER TABLE content_policy DROP COLUMN cooldown;
