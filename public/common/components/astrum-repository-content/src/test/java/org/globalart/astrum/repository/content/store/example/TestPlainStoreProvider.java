
package org.globalart.astrum.repository.content.store.example;

import org.globalart.astrum.repository.content.browse.store.example.TestBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Plain store module for a plain format that uses standard stores with the format's DAOs.
 */
@Qualifier("test")
public class TestPlainStoreProvider
    extends
    FormatStoreSupport<TestContentRepositoryDAO, TestComponentDAO, TestAssetDAO, TestAssetBlobDAO, TestBrowseNodeDAO>
{
  // nothing to add...
}
