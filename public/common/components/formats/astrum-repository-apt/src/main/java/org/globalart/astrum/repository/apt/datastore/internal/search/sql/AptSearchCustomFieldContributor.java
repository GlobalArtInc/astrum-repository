
package org.globalart.astrum.repository.apt.datastore.internal.search.sql;

import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
@Qualifier(AptFormat.NAME)
public class AptSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public void populateSearchCustomFields(final SearchRecord searchTableData, final Asset asset) {
    searchTableData.addKeyword(asset.path().substring(1));
  }
}
