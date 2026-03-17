
package org.globalart.astrum.content.example.internal.store;

import org.globalart.astrum.content.example.internal.recipe.ExampleFormat;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Configures the content store bindings for an 'example' format.
 */
@Qualifier(ExampleFormat.NAME)
public class ExampleStoresProvider
    extends
    FormatStoreSupport<ExampleContentRepositoryDAO, ExampleComponentDAO, ExampleAssetDAO, ExampleAssetBlobDAO, ExampleBrowseNodeDAO>
{
  // nothing to add...
}
