
package org.globalart.astrum.repository.rest.api;

import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;

import org.springframework.stereotype.Component;

/**
 * Descriptor for the {@link AssetXO} class
 *
 * @since 3.29
 */
@Component
@FunctionalInterface
public interface AssetXODescriptor
{
  Set<String> listExposedAttributeKeys();

  default Object transformAttributeValue(final String key, final Object value) {
    return value;
  }

  @Nullable
  default String computeRegistryUrl(final Repository repository) {
    return null;
  }
}
