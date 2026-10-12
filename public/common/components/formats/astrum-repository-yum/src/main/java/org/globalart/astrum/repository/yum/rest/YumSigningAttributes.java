package org.globalart.astrum.repository.yum.rest;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.repository.yum.internal.YumSigningFacet.CONFIG_KEY;
import static org.globalart.astrum.repository.yum.internal.YumSigningFacet.KEYPAIR;
import static org.globalart.astrum.repository.yum.internal.YumSigningFacet.PASSPHRASE;

@JsonInclude(Include.NON_NULL)
public class YumSigningAttributes
{
  @Schema(description = "Armored PGP private key used to sign repodata/repomd.xml (gpg --export-secret-key --armor)",
      example = "")
  private final String keypair;

  @Schema(description = "Passphrase of the signing key", example = "")
  private final String passphrase;

  @JsonCreator
  public YumSigningAttributes(
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
  public static YumSigningAttributes of(final Repository repository) {
    Map<String, Object> signing = repository.getConfiguration().getAttributes().get(CONFIG_KEY);
    if (signing == null || !(signing.get(KEYPAIR) instanceof String keypair) || keypair.isBlank()) {
      return null;
    }
    return new YumSigningAttributes(keypair, null);
  }

  public static void apply(
      final Configuration configuration,
      @Nullable final YumSigningAttributes signing,
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
          && previousKeypair.strip().equals(keypair) && previous.get(PASSPHRASE) instanceof String previousPassphrase) {
        passphrase = previousPassphrase;
      }
    }
    NestedAttributesMap attributes = configuration.attributes(CONFIG_KEY);
    attributes.set(KEYPAIR, keypair);
    if (passphrase != null && !passphrase.isEmpty()) {
      attributes.set(PASSPHRASE, passphrase);
    }
  }
}
