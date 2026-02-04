
package org.globalart.astrum.repository.content;

import java.time.OffsetDateTime;

import org.globalart.astrum.common.collect.NestedAttributesMap;

/**
 * Metadata common to all repository content.
 *
 * @since 3.20
 */
public interface RepositoryContent
{
  /**
   * Schemaless content attributes.
   */
  NestedAttributesMap attributes();

  /**
   * Shortcut to content sub-attributes.
   */
  default NestedAttributesMap attributes(String key) {
    return attributes().child(key);
  }

  /**
   * When the metadata was first created.
   */
  OffsetDateTime created();

  /**
   * When the metadata was last updated.
   */
  OffsetDateTime lastUpdated();
}
