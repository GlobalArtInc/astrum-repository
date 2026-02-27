
package org.globalart.astrum.repository.upload;

import java.io.InputStream;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

/**
 * Factory for creating {@link TempBlob}
 *
 * @since 3.24
 */
public interface TempBlobFactory
{
  TempBlob create(Repository repository, InputStream inputStream, Iterable<HashAlgorithm> hashAlgorithms);

  TempBlob create(Repository repository, Payload payload, Iterable<HashAlgorithm> hashAlgorithms);
}
