package org.globalart.astrum.repository.yum.internal.search;

import java.util.Map;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.search.sql.SearchRecord;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(YumFormat.NAME)
public class YumSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    if (!(asset.attributes().get(YumFormat.NAME) instanceof Map<?, ?> attributes)) {
      return;
    }
    if (attributes.get(RpmPackage.VERSION) instanceof String version) {
      searchRecord.addFormatFieldValue1(version, true);
    }
    if (attributes.get(RpmPackage.RELEASE) instanceof String release) {
      searchRecord.addFormatFieldValue4(release, true);
    }
    if (attributes.get(RpmPackage.ARCHITECTURE) instanceof String architecture) {
      searchRecord.addFormatFieldValue6(architecture, true);
      searchRecord.addKeyword(architecture);
    }
    if (attributes.get(RpmPackage.SUMMARY) instanceof String summary) {
      searchRecord.addKeyword(summary);
    }
  }
}
