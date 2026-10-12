package org.globalart.astrum.repository.gitlfs.internal;

import java.util.Optional;
import java.util.OptionalLong;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

/**
 * Stores Git LFS objects. Each object is a component named by its id with a single asset.
 */
@Facet.Exposed
public interface GitLfsContentFacet
    extends ContentFacet
{
  String OBJECT_KIND = "OBJECT";

  Optional<FluentAsset> getObject(String oid);

  /**
   * @return the size of the stored object, empty when the repository does not hold it
   */
  OptionalLong size(String oid);

  /**
   * Stores the payload as the object with the given id.
   *
   * @throws GitLfsException with 422 when the SHA-256 of the payload is not the object id
   */
  FluentAsset put(String oid, Payload payload);

  TempBlob ingest(Payload payload);

  /**
   * @return whether the write policy of the repository forbids storing objects
   */
  boolean isReadOnly();

  /**
   * Stores an ingested blob under the object id it hashes to; an object the repository already holds is kept.
   */
  FluentAsset put(TempBlob blob);
}
