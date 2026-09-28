
package com.globalart.astrum.ssl.plugin.spi;

import org.globalart.astrum.capability.CapabilityReference;

/**
 * Manages retrieve / update of TrustStore capabilities / type.
 *
 * @since ssl 1.0
 */
public interface CapabilityManager
{

  CapabilityReference get(String id);

  CapabilityReference enable(String id, boolean enabled) throws Exception;

}
