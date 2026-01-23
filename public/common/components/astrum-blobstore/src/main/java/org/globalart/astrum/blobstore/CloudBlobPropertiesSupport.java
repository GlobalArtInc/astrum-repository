
package org.globalart.astrum.blobstore;

import java.io.ByteArrayOutputStream;
import java.io.IOException;

import org.globalart.astrum.common.property.ImplicitSourcePropertiesFile;

/**
 * Support class which allows properties of CloudBlobs to specify metadata which should be used to indicate whether that
 * properties file is for a TempBlob
 *
 * @param <T> The metadata type for that cloud blob store e.g.
 *          <code>software.amazon.awssdk.services.s3.model.PutObjectRequest.Builder</code>
 *          for AWS S3 BlobStore implementation.
 * @see CloudBlobStoreSupport
 * @since 3.37
 */
public abstract class CloudBlobPropertiesSupport<T>
    extends ImplicitSourcePropertiesFile
{
  /**
   * An instance of the metadata type for that particular blob store implementation. E.g for the S3 blob store, this
   * would be an instance of <code>software.amazon.awssdk.services.s3.model.PutObjectRequest.Builder</code> which would
   * contain a key value
   * pair which indicates whether or not the blob associated with the property file being written is a TempBlob or not.
   * Thus, subclasses should use this metadata instance to specify whether a blob properties file is for a blob which is
   * still temporary i.e. a TempBlob.
   */
  public abstract T getMetadata();

  /**
   * A {@link ByteArrayOutputStream} containing the blob properties to be written.
   */
  public abstract ByteArrayOutputStream getData() throws IOException;

  /**
   * Writes the data and associated metadata to the cloud destination (e.g. AWS S3, Google Storage, Azure Blob etc).
   */
  protected abstract void write(final ByteArrayOutputStream data, final T metadata);

  @Override
  public void store() throws IOException {
    ByteArrayOutputStream data = getData();
    T metadata = getMetadata();
    write(data, metadata);
  }
}
