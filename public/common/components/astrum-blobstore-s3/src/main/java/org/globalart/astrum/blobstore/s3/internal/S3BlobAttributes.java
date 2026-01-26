
package org.globalart.astrum.blobstore.s3.internal;

import java.io.IOException;
import java.util.Map;

import org.globalart.astrum.blobstore.BlobAttributesSupport;
import org.globalart.astrum.blobstore.api.BlobAttributes;
import org.globalart.astrum.blobstore.api.BlobMetrics;

import software.amazon.awssdk.services.s3.model.NoSuchKeyException;
import software.amazon.awssdk.services.s3.model.S3Exception;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link BlobAttributes} backed by {@link S3PropertiesFile}
 *
 * @since 3.6.1
 */
public class S3BlobAttributes
    extends BlobAttributesSupport<S3PropertiesFile>
{
  public S3BlobAttributes(final EncryptingS3Client s3, final String bucket, final String key) {
    super(new S3PropertiesFile(s3, bucket, key), null, null);
  }

  public S3BlobAttributes(
      final EncryptingS3Client s3,
      final String bucket,
      final String key,
      final Map<String, String> headers,
      final BlobMetrics metrics)
  {
    super(new S3PropertiesFile(s3, bucket, key), checkNotNull(headers), checkNotNull(metrics));
  }

  public boolean load() throws IOException {
    try {
      propertiesFile.load();
      readFrom(propertiesFile);
      return true;
    }
    catch (NoSuchKeyException e) {
      // AWS SDK 2.x throws NoSuchKeyException for missing objects
      return false;
    }
    catch (S3Exception e) {
      if (e.statusCode() != 404) {
        throw e;
      }
      return false;
    }
  }

  @Override
  public void store() throws IOException {
    writeTo(propertiesFile);
    propertiesFile.store();
  }

  @Override
  public void writeProperties() {
    writeTo(propertiesFile);
  }
}
