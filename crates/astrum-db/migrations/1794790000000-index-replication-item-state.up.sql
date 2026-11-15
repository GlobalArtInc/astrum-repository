CREATE INDEX replication_item_state ON replication_item (replication_id, state);
CREATE INDEX replication_item_queued ON replication_item (replication_id, first_queued_at)
    WHERE state IN ('pending', 'leased');
