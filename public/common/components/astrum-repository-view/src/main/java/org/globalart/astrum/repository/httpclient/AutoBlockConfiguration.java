
package org.globalart.astrum.repository.httpclient;

/**
 * Provides format specific configuration for what to auto-block.
 * 
 * @since 3.14
 */
public interface AutoBlockConfiguration
{
  boolean shouldBlock(int statusCode);
}
