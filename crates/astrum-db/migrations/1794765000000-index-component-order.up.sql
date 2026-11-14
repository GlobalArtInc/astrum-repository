CREATE INDEX component_listing_order ON component
    (repository, namespace COLLATE "C", name COLLATE "C", version COLLATE "C");
