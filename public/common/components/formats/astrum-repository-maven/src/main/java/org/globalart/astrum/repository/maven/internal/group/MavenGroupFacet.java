
package org.globalart.astrum.repository.maven.internal.group;

import java.io.IOException;
import java.io.OutputStream;
import java.util.LinkedHashMap;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.thread.io.StreamCopier;

@Facet.Exposed
public interface MavenGroupFacet
    extends GroupFacet
{
  /**
   * Fetches cached content if exists, or {@code null}.
   */
  @Nullable
  Content getCached(MavenPath mavenPath) throws IOException;

  /**
   * Merges and caches and returns the merged metadata. Returns {@code null} if no usable response was in passed in
   * map.
   */
  @Nullable
  Content mergeAndCache(MavenPath mavenPath, Map<Repository, Response> responses) throws IOException;

  /**
   * Merges the metadata but doesn't cache it. Returns {@code null} if no usable response was in passed in map.
   *
   * @since 3.13
   */
  @Nullable
  Content mergeWithoutCaching(MavenPath mavenPath, Map<Repository, Response> responses) throws IOException;

  @FunctionalInterface
  interface ContentFunction<T>
  {
    Content apply(T data, String contentType) throws IOException;
  }

  /**
   * Allows different merge methods to be used with the {@link StreamCopier}
   */
  interface MetadataMerger
  {
    void merge(OutputStream outputStream, MavenPath mavenPath, LinkedHashMap<Repository, Content> contents);
  }
}
