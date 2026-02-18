
package org.globalart.astrum.repository.manager.internal;

import java.util.Map;

/**
 * Interface for encoding/decoding repository secrets in configuration attributes.
 * Implementations handle format-specific secret encryption (e.g., Docker ECR credentials).
 */
public interface SecretEncoder
{
  /**
   * Encode secrets in repository attributes before storage.
   *
   * @param attributes repository configuration attributes
   */
  void encodeSecrets(Map<String, Map<String, Object>> attributes);

  /**
   * Encode secrets when updating repository, comparing old and new attributes.
   *
   * @param oldAttributes previous configuration attributes
   * @param newAttributes new configuration attributes
   */
  void encodeSecrets(
      Map<String, Map<String, Object>> oldAttributes,
      Map<String, Map<String, Object>> newAttributes);

  /**
   * Remove secrets from secret store when repository is deleted.
   *
   * @param attributes repository configuration attributes
   */
  void removeSecret(Map<String, Map<String, Object>> attributes);

  /**
   * Remove old secrets when repository is updated.
   *
   * @param oldAttributes previous configuration attributes
   * @param newAttributes new configuration attributes
   */
  void removeSecret(
      Map<String, Map<String, Object>> oldAttributes,
      Map<String, Map<String, Object>> newAttributes);
}
