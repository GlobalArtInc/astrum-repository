CREATE INDEX asset_protobuf_id ON asset (repository, (attributes #>> '{protobuf,id}'))
    WHERE format = 'protobuf';
