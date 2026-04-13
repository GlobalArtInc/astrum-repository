
package org.globalart.astrum.repository.apt.rest;

import jakarta.validation.constraints.NotEmpty;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * @since 3.20
 */
public class AptSigningRepositoriesAttributes
{
  @NotEmpty
  @Schema(description = "PGP signing key pair (armored private key e.g. gpg --export-secret-key --armor)",
      example = "")
  private final String keypair;

  @Schema(description = "Passphrase to access PGP signing key", example = "")
  private final String passphrase;

  @JsonCreator
  public AptSigningRepositoriesAttributes(
      @JsonProperty("keypair") final String keypair,
      @JsonProperty("passphrase") final String passphrase)
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
}
