package org.globalart.astrum.repository.terraform.internal.store;

import org.globalart.astrum.repository.content.store.FormatStoreSupport;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.internal.browse.TerraformBrowseNodeDAO;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(TerraformFormat.NAME)
public class TerraformStoresProvider
    extends FormatStoreSupport<TerraformContentRepositoryDAO, TerraformComponentDAO, TerraformAssetDAO, TerraformAssetBlobDAO, TerraformBrowseNodeDAO>
{
}
