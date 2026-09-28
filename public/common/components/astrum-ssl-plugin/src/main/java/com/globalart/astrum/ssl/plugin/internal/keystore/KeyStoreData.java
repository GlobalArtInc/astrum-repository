
package com.globalart.astrum.ssl.plugin.internal.keystore;

import org.globalart.astrum.common.entity.HasName;

/**
 * {@link java.security.KeyStore} data.
 *
 * @since 3.21
 */
public class KeyStoreData
    implements HasName
{
  private String name;

  private byte[] bytes;

  @Override
  public String getName() {
    return name;
  }

  @Override
  public void setName(final String name) {
    this.name = name;
  }

  public byte[] getBytes() { // NOSONAR
    return bytes; // NOSONAR: this is just a temporary transfer object
  }

  public void setBytes(final byte[] bytes) { // NOSONAR
    this.bytes = bytes; // NOSONAR: this is just a temporary transfer object
  }
}
