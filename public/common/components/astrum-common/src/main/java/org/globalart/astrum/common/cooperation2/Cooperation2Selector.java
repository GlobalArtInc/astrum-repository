
package org.globalart.astrum.common.cooperation2;

/**
 * Selects the proper cooperation2 implementation to use based on the current environment.
 */
public interface Cooperation2Selector
{
  Cooperation2Factory select();
}
