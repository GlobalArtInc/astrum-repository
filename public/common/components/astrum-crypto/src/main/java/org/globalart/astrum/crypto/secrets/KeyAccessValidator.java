
package org.globalart.astrum.crypto.secrets;

/**
 * Validator used to check if a key is accessible to all nodes in the cluster before using it
 */
public interface KeyAccessValidator
{
  /**
   * Checks the provided key is accessible to all nodes in the cluster
   *
   * @param keyId the key id to check
   * @return {@code true} if the key is accessible, {@code false} otherwise
   */
  boolean isValidKey(String keyId);
}
