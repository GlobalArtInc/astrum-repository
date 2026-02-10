
package org.globalart.astrum.repository.content.upload.internal;

import java.io.InputStream;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.upload.TempBlobFactory;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Content implementation of {@code TempBlobFactory}
 *
 * @since 3.24
 */
@Component
public class TembBlobFactoryImpl
    implements TempBlobFactory
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public TempBlob create(
      final Repository repository,
      final InputStream inputStream,
      final Iterable<HashAlgorithm> hashAlgorithms)
  {
    return repository.facet(ContentFacet.class).blobs().ingest(inputStream, null, hashAlgorithms);
  }

  @Override
  public TempBlob create(
      final Repository repository,
      final Payload payload,
      final Iterable<HashAlgorithm> hashAlgorithms)
  {
    return repository.facet(ContentFacet.class).blobs().ingest(payload, hashAlgorithms);
  }
}
