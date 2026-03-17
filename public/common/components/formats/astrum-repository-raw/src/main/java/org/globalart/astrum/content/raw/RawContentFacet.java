
package org.globalart.astrum.content.raw;

import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA512;

/**
 * Provides persistent content for the 'raw' format.
 *
 * @since 3.24
 */
@Facet.Exposed
public interface RawContentFacet
    extends ContentFacet
{
  /**
   * Assets stored prior to version 3.68 may only have MD5 and SHA1 hashes stored.
   */
  public final Iterable<HashAlgorithm> HASHING = ImmutableList.of(MD5, SHA1, SHA256, SHA512);

  Optional<Content> get(String path) throws IOException;

  FluentAsset getOrCreateAsset(Repository repository, String componentName, String componentGroup, String assetName);

  Content put(String path, Payload content) throws IOException;

  boolean delete(String path) throws IOException;
}
