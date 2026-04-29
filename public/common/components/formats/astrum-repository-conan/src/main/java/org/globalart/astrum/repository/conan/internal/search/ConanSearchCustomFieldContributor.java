package org.globalart.astrum.repository.conan.internal.search;

import java.util.Map;

import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(ConanFormat.NAME)
public class ConanSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    asset.component().ifPresent(component -> {
      if (component.attributes().get(ConanFormat.NAME) instanceof Map<?, ?> attributes) {
        if (attributes.get("baseVersion") instanceof String baseVersion) {
          searchRecord.addFormatFieldValue1(baseVersion);
        }
        if (attributes.get("channel") instanceof String channel) {
          searchRecord.addFormatFieldValue2(channel);
        }
      }
    });
    if (!(asset.attributes().get(ConanFormat.NAME) instanceof Map<?, ?> attributes)) {
      return;
    }
    if (attributes.get("recipeRevision") instanceof String revision) {
      searchRecord.addFormatFieldValue3(revision);
    }
    if (attributes.get("packageId") instanceof String packageId) {
      searchRecord.addFormatFieldValue4(packageId);
      searchRecord.addKeyword(packageId);
    }
    if (attributes.get("packageRevision") instanceof String packageRevision) {
      searchRecord.addFormatFieldValue5(packageRevision);
    }
  }
}
