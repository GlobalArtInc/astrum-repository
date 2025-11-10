
package org.globalart.astrum.crypto.secrets;

import org.globalart.astrum.crypto.internal.error.CipherException;

import com.fasterxml.jackson.annotation.JsonIgnoreType;

/**
 * An application secret (e.g. a password persisted with reversible encryption.
 * <p>
 * Marked as {@link JsonIgnoreType} to prevent serialization in case of accidental return.
 */
@JsonIgnoreType
public interface Secret
{
  /**
   * May have latency due to decryption or external secret management.
   */
  char[] decrypt() throws CipherException;

  /**
   * Returns the token id. Note, if the system has not been upgraded this may be a legacy encrypted string for
   * compatibility.
   */
  String getId();
}
