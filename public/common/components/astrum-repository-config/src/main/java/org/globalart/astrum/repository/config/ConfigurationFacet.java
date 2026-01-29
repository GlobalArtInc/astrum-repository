
package org.globalart.astrum.repository.config;

import jakarta.validation.ValidationException;

import org.globalart.astrum.repository.Facet;

/**
 * Configuration {@link Facet}.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface ConfigurationFacet
    extends Facet
{
  /**
   * Persist configuration.
   */
  void save() throws Exception;

  /**
   * Convert value to given type.
   */
  <T> T convert(Object value, Class<T> type);

  /**
   * Read object of given type from named configuration section.
   */
  <T> T readSection(Configuration configuration, String section, Class<T> type);

  /**
   * Validate given object.
   *
   * @throws ValidationException
   */
  void validate(Object value, Class<?>... groups);

  /**
   * Read and validate object from named configuration section.
   */
  <T> T validateSection(Configuration configuration, String section, Class<T> type, Class<?>... groups);
}
