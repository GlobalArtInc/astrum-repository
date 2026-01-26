
package org.globalart.astrum.blobstore.s3.internal;

import java.io.InputStream;

import org.globalart.astrum.blobstore.api.BlobStoreException;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Uploads a file with the TransferManager.
 *
 * @since 3.7
 * @deprecated replaced with {@link MultipartUploader}
 */
@ConditionalOnProperty(name = "astrum.s3.uploaderName", havingValue = "transfer-manager-uploader")
@Deprecated
@Component
@Qualifier("transfer-manager-uploader")
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TransferManagerUploader
    implements S3Uploader
{
  @Override
  public void upload(
      final EncryptingS3Client s3,
      final String bucket,
      final String key,
      final InputStream contents)
  {
    try {
      s3.uploadWithTransferManger(bucket, key, contents);
    }
    catch (Exception e) {
      throw new BlobStoreException("error uploading blob", e, null);
    }
  }
}
