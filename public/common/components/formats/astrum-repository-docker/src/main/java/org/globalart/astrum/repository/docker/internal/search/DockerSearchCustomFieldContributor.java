package org.globalart.astrum.repository.docker.internal.search;

import java.util.function.Consumer;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerFormat.NAME)
public class DockerSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    if (!DockerContentFacet.KIND_TAG.equals(asset.kind())) {
      return;
    }
    NestedAttributesMap attributes = asset.attributes(DockerFormat.NAME);
    add(attributes, DockerContentFacet.ATTR_DIGEST, searchRecord::addFormatFieldValue1);
    add(attributes, DockerContentFacet.ATTR_MEDIA_TYPE, searchRecord::addFormatFieldValue2);
    add(attributes, DockerContentFacet.ATTR_ARTIFACT_TYPE, searchRecord::addFormatFieldValue3);
    add(attributes, DockerContentFacet.ATTR_ARTIFACT_KIND, searchRecord::addFormatFieldValue4);
  }

  private static void add(final NestedAttributesMap attributes, final String key, final Consumer<String> target) {
    if (attributes.get(key) instanceof String value && !value.isBlank()) {
      target.accept(value);
    }
  }
}
