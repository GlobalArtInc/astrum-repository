package org.globalart.astrum.repository.p2.internal.search;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.PLUGIN_NAME;

@Component
@Qualifier(P2Format.NAME)
public class P2SearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    Object pluginName = asset.attributes(P2Format.NAME).get(PLUGIN_NAME);
    if (pluginName instanceof String name && !name.isBlank()) {
      searchRecord.addFormatFieldValue1(name);
    }
  }
}
