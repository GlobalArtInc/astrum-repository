
package org.globalart.astrum.internal.capability;

/**
 * Factory of {@link ValidityConditionHandler}.
 *
 * @since capabilities 2.0
 */
public interface ValidityConditionHandlerFactory
{

  ValidityConditionHandler create(final DefaultCapabilityReference reference);

}
