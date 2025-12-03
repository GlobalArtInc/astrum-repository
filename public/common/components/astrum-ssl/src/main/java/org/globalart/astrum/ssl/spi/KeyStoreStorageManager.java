
package org.globalart.astrum.ssl.spi;

/**
 * Manager for accessing {@link KeyStoreStorage} implementations.
 * 
 * @since 3.1
 */
public interface KeyStoreStorageManager
{
  /**
   * Creates a persistent storage for the specified key store name.
   * 
   * @return The requested storage, never {@code null}.
   */
  KeyStoreStorage createStorage(String keyStoreName);
}
