
package org.globalart.astrum.content.raw.internal.store;

import org.globalart.astrum.content.raw.internal.browse.RawBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.raw.internal.RawFormat;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Configures the content store bindings for the raw format.
 */
@Qualifier(RawFormat.NAME)
public class RawStoresProvider
    extends FormatStoreSupport<RawContentRepositoryDAO, RawComponentDAO, RawAssetDAO, RawAssetBlobDAO, RawBrowseNodeDAO>
{
  // nothing to add...
}
