
package org.globalart.astrum.api.rest.common.blobstore.s3.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

import static com.fasterxml.jackson.annotation.JsonInclude.Include.NON_NULL;

/**
 * Encapsulates the encryption type and key to use for encrypting an s3 blob store at rest i.e AWS S3 server side
 * encryption.
 *
 * @since 3.20
 */
@JsonInclude(NON_NULL)
public class S3BlobStoreApiEncryption
{
  @Schema(description = "The type of S3 server side encryption to use.")
  private final String encryptionType;

  @Schema(description = "The encryption key.")
  private final String encryptionKey;

  @JsonCreator
  public S3BlobStoreApiEncryption(
      @JsonProperty("encryptionType") final String encryptionType,
      @JsonProperty("encryptionKey") final String encryptionKey)
  {
    this.encryptionType = encryptionType;
    this.encryptionKey = encryptionKey;
  }

  public String getEncryptionType() {
    return encryptionType;
  }

  public String getEncryptionKey() {
    return encryptionKey;
  }
}
