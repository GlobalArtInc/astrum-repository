package org.globalart.astrum.repository.nuget.internal.search;

import java.util.Map;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    if (!(asset.attributes().get(NugetFormat.NAME) instanceof Map<?, ?> attributes)) {
      return;
    }
    if (attributes.get(NugetPackageMetadata.ID) instanceof String id) {
      searchRecord.addKeyword(id);
    }
    if (attributes.get(NugetPackageMetadata.TAGS) instanceof String tags && !tags.isBlank()) {
      for (String tag : tags.trim().split("[\\s,;]+")) {
        searchRecord.addFormatFieldValue1(tag);
        searchRecord.addKeyword(tag);
      }
    }
  }
}
