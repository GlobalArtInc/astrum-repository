CREATE INDEX asset_nix_nar ON asset (repository, (attributes #>> '{nix,nar}'))
    WHERE format = 'nix';
