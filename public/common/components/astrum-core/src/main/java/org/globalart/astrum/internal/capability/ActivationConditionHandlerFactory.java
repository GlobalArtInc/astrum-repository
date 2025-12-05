
package org.globalart.astrum.internal.capability;

/**
 * Factory of {@link ActivationConditionHandler}.
 *
 * @since capabilities 2.0
 */
public interface ActivationConditionHandlerFactory
{

  ActivationConditionHandler create(final DefaultCapabilityReference reference);

}
