
package org.globalart.astrum.repository.apt.api;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model for apt repository signing.
 *
 * @since 3.20
 */
@JsonInclude(Include.NON_NULL)
public class AptSigningRepositoriesAttributes
{
  @Schema(description = "PGP signing key pair (armored private key e.g. gpg --export-secret-key --armor)",
      example = "")

  protected final String keypair;

  @Schema(description = "Passphrase to access PGP signing key", example = "")
  protected final String passphrase;

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
