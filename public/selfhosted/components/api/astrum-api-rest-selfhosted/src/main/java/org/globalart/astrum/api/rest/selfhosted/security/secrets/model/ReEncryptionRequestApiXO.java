
package org.globalart.astrum.api.rest.selfhosted.security.secrets.model;

import javax.annotation.Nullable;
import jakarta.validation.constraints.NotBlank;

import io.swagger.v3.oas.annotations.media.Schema;

public class ReEncryptionRequestApiXO
{
  @Schema(description = "Key identifier that will be used to re-encrypt secrets")
  @NotBlank
  private String secretKeyId;

  @Schema(description = "Optional - Email to notify when task finishes")
  @Nullable
  private String notifyEmail;

  public ReEncryptionRequestApiXO() {
    // serialization
  }

  public ReEncryptionRequestApiXO(final String secretKeyId, final String notifyEmail) {
    this.secretKeyId = secretKeyId;
    this.notifyEmail = notifyEmail;
  }

  public String getSecretKeyId() {
    return secretKeyId;
  }

  @Nullable
  public String getNotifyEmail() {
    return notifyEmail;
  }
}
