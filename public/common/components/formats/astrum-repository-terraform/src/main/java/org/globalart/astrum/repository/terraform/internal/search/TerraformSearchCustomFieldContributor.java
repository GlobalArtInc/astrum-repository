package org.globalart.astrum.repository.terraform.internal.search;

import java.util.function.Consumer;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.search.sql.SearchRecord;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    NestedAttributesMap attributes = asset.attributes(TerraformFormat.NAME);
    add(attributes, TerraformContentFacet.NAMESPACE, searchRecord::addFormatFieldValue1, searchRecord);
    add(attributes, TerraformContentFacet.NAME, searchRecord::addFormatFieldValue2, searchRecord);
    add(attributes, TerraformContentFacet.PROVIDER, searchRecord::addFormatFieldValue3, searchRecord);
    add(attributes, TerraformContentFacet.KIND, searchRecord::addFormatFieldValue4, null);
  }

  private static void add(
      final NestedAttributesMap attributes,
      final String key,
      final Consumer<String> target,
      final SearchRecord keywords)
  {
    if (attributes.get(key) instanceof String value && !value.isBlank()) {
      target.accept(value);
      if (keywords != null) {
        keywords.addKeyword(value);
      }
    }
  }
}
