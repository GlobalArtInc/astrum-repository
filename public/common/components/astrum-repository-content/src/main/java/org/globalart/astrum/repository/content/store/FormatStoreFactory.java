
package org.globalart.astrum.repository.content.store;

/**
 * Factory that creates content stores for a format-specific DAO.
 *
 * @since 3.26
 */
@SuppressWarnings({"rawtypes"})
interface FormatStoreFactory
{
  <T extends ContentStoreSupport> T createFormatStore(final String contentStoreName);
}
