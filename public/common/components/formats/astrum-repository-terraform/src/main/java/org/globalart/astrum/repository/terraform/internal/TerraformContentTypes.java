package org.globalart.astrum.repository.terraform.internal;

import org.globalart.astrum.repository.terraform.AssetKind;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_OCTET_STREAM;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

public final class TerraformContentTypes
{
  public static final String PGP_SIGNATURE = "application/pgp-signature";

  public static final String PGP_KEYS = "application/pgp-keys";

  private TerraformContentTypes() {
  }

  public static String forKind(final String kind) {
    AssetKind assetKind;
    try {
      assetKind = AssetKind.valueOf(kind);
    }
    catch (IllegalArgumentException | NullPointerException e) {
      return APPLICATION_OCTET_STREAM;
    }
    return switch (assetKind) {
      case MODULE_VERSIONS, PROVIDER_VERSIONS, PROVIDER_DOWNLOAD, PROVIDER_MANIFEST -> APPLICATION_JSON;
      case PROVIDER_SHASUMS -> TEXT_PLAIN;
      case PROVIDER_SIGNATURE -> PGP_SIGNATURE;
      case PROVIDER_KEY -> PGP_KEYS;
      default -> APPLICATION_OCTET_STREAM;
    };
  }
}
