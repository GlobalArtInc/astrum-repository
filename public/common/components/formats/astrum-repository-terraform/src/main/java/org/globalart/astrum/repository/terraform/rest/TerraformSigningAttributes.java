package org.globalart.astrum.repository.terraform.rest;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.repository.terraform.internal.signing.TerraformSigningFacet.CONFIG_KEY;
import static org.globalart.astrum.repository.terraform.internal.signing.TerraformSigningFacet.KEYPAIR;
import static org.globalart.astrum.repository.terraform.internal.signing.TerraformSigningFacet.PASSPHRASE;

/**
 * The OpenPGP key a hosted repository signs provider checksums with. The passphrase is write-only: it is never
 * returned, and an update without one keeps the saved passphrase while the key stays the same.
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public class TerraformSigningAttributes
{
  @Schema(description = "ASCII-armored OpenPGP secret key (gpg --armor --export-secret-keys) used to sign the "
      + "SHA256SUMS of provider versions uploaded without a signature", example = "")
  private final String keypair;

  @Schema(description = "Passphrase of the secret key; write-only", example = "")
  private final String passphrase;

  @JsonCreator
  public TerraformSigningAttributes(
      @JsonProperty(KEYPAIR) final String keypair,
      @JsonProperty(PASSPHRASE) final String passphrase)
  {
    this.keypair = keypair;
    this.passphrase = passphrase;
  }

  public String getKeypair() {
    return keypair;
  }

  public String getPassphrase() {
    return passphrase;
  }

  @Nullable
  public static TerraformSigningAttributes of(final Repository repository) {
    Map<String, Object> signing = repository.getConfiguration().getAttributes().get(CONFIG_KEY);
    if (signing == null || !(signing.get(KEYPAIR) instanceof String keypair) || keypair.isBlank()) {
      return null;
    }
    return new TerraformSigningAttributes(keypair, null);
  }

  public static void apply(
      final Configuration configuration,
      @Nullable final TerraformSigningAttributes signing,
      @Nullable final Repository existing)
  {
    if (signing == null || signing.getKeypair() == null || signing.getKeypair().isBlank()) {
      return;
    }
    String keypair = signing.getKeypair().strip();
    String passphrase = signing.getPassphrase();
    if ((passphrase == null || passphrase.isEmpty()) && existing != null) {
      Map<String, Object> previous = existing.getConfiguration().getAttributes().get(CONFIG_KEY);
      if (previous != null && previous.get(KEYPAIR) instanceof String previousKeypair
          && previousKeypair.strip().equals(keypair) && previous.get(PASSPHRASE) instanceof String saved) {
        passphrase = saved;
      }
    }
    NestedAttributesMap attributes = configuration.attributes(CONFIG_KEY);
    attributes.set(KEYPAIR, keypair);
    if (passphrase != null && !passphrase.isEmpty()) {
      attributes.set(PASSPHRASE, passphrase);
    }
  }
}
