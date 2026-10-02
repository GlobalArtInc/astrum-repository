
package org.globalart.astrum.api.rest.selfhosted.security.apikey.model;

import javax.annotation.Nullable;

import io.swagger.v3.oas.annotations.media.Schema;

public class ApiKeysReEncryptionRequestApiXO
{
  @Schema(description = "Optional - The algorithm to be used to decrypt the principals")
  @Nullable
  private String algorithmForDecryption;

  @Schema(description = "Optional - The iterations to be used to decrypt the principals")
  @Nullable
  private Integer iterationsForDecryption;

  @Schema(description = "Optional - Email to notify when task finishes")
  @Nullable
  private String notifyEmail;

  public ApiKeysReEncryptionRequestApiXO() {
    // serialization
  }

  public ApiKeysReEncryptionRequestApiXO(
      @Nullable final String algorithm,
      @Nullable final Integer iterations,
      @Nullable final String notifyEmail)
  {
    this.algorithmForDecryption = algorithm;
    this.iterationsForDecryption = iterations;
    this.notifyEmail = notifyEmail;
  }

  @Nullable
  public String getAlgorithmForDecryption() {
    return algorithmForDecryption;
  }

  @Nullable
  public Integer getIterationsForDecryption() {
    return iterationsForDecryption;
  }

  @Nullable
  public String getNotifyEmail() {
    return notifyEmail;
  }
}
