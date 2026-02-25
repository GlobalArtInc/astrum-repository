
package org.globalart.astrum.repository.search.sql;

import java.time.OffsetDateTime;
import java.util.List;

import org.globalart.astrum.common.collect.NestedAttributesMap;

/**
 * Provides search results.
 *
 * @since 3.41
 */
public interface SearchResult
{
  /**
   * When the metadata was first created.
   */
  OffsetDateTime lastModified();

  Integer componentId();

  Integer repositoryId();

  String format();

  String namespace();

  String componentName();

  String repositoryName();

  String version();

  String normalisedVersion();

  List<String> tags();

  /**
   * Schemaless content attributes.
   */
  NestedAttributesMap attributes();
}
