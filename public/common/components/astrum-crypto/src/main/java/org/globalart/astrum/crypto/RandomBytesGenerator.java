
package org.globalart.astrum.crypto;

/**
 * Generates random bytes of a specific size.
 *
 * @since 3.0
 */
public interface RandomBytesGenerator
{
  byte[] generate(final int size);
}
