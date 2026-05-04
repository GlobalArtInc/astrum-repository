package org.globalart.astrum.repository.cargo.internal.store;

import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.internal.browse.CargoBrowseNodeDAO;
import org.globalart.astrum.repository.content.store.FormatStoreSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(CargoFormat.NAME)
public class CargoStoresProvider
    extends FormatStoreSupport<CargoContentRepositoryDAO, CargoComponentDAO, CargoAssetDAO, CargoAssetBlobDAO, CargoBrowseNodeDAO>
{
}
