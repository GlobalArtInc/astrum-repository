
package org.globalart.astrum.repository.security;

/**
 * Manages {@link VariableResolverAdapter} implementations.
 * 
 * @since 3.1
 *
 */
public interface VariableResolverAdapterManager
{
  /**
   * Gets a {@link VariableResolverAdapter} for the specified format. If no specialized implemention for the format
   * exists, a default implementation will be used.
   */
  <V extends VariableResolverAdapter> V get(String format);
}
