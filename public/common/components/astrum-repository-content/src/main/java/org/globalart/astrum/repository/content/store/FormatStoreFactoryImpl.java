
package org.globalart.astrum.repository.content.store;

import jakarta.inject.Provider;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Factory that creates content stores for a format-specific DAO.
 *
 * @since 3.26
 */
@SuppressWarnings({"rawtypes", "unchecked"})
class FormatStoreFactoryImpl
    implements FormatStoreFactory
{
  private final Provider<ContentStoreFactory> factoryProvider;

  private final Class<?> formatDaoClass;

  FormatStoreFactoryImpl(final Provider<ContentStoreFactory> factoryProvider, final Class<?> formatDaoClass) {
    this.factoryProvider = checkNotNull(factoryProvider);
    this.formatDaoClass = checkNotNull(formatDaoClass);
  }

  @Override
  public <T extends ContentStoreSupport> T createFormatStore(final String contentStoreName) {
    return (T) factoryProvider.get().createContentStore(contentStoreName, formatDaoClass);
  }
}
