
package org.globalart.astrum.repository.content.store;

import jakarta.inject.Provider;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Factory that creates content stores for a format-specific DAO.
 */
@SuppressWarnings({"unchecked"})
class FormatStoreFactorySpringImpl<STORE extends ContentStoreSupport<?>>
    implements FormatStoreFactory
{
  private final Provider<STORE> factoryProvider;

  FormatStoreFactorySpringImpl(
      final Provider<STORE> factoryProvider)
  {
    this.factoryProvider = checkNotNull(factoryProvider);
  }

  @Override
  public STORE createFormatStore(final String contentStoreName) {
    return factoryProvider.get();
  }
}
