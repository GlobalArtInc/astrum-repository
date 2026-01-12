
package org.globalart.astrum.capability;

/**
 * To be implemented by {@link Condition}s that needs the capability context of capability they are used for.
 */
public interface CapabilityContextAware
{

  CapabilityContextAware setContext(CapabilityContext context);

}
