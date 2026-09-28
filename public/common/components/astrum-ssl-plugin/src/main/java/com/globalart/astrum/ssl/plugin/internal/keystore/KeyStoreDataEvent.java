
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.security.KeyStore;

import org.globalart.astrum.common.entity.EntityVersion;

/**
 * Event sent out when {@link KeyStore} data changes.
 *
 * @since 3.1
 */
public interface KeyStoreDataEvent
{
  boolean isLocal();

  String getRemoteNodeId();

  EntityVersion getVersion();

  String getKeyStoreName();
}
