package org.globalart.astrum.repository.rubygems.internal.search;

import java.util.function.Consumer;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.RubygemsContentFacet;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(RubygemsFormat.NAME)
public class RubygemsSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    NestedAttributesMap attributes = asset.attributes(RubygemsContentFacet.ATTRIBUTES);
    add(attributes, GemSpec.PLATFORM, searchRecord::addFormatFieldValue1);
    add(attributes, GemSpec.SUMMARY, searchRecord::addFormatFieldValue2);
    add(attributes, GemSpec.DESCRIPTION, searchRecord::addFormatFieldValue3);
  }

  private static void add(final NestedAttributesMap attributes, final String key, final Consumer<String> target) {
    Object value = attributes.get(key);
    if (value instanceof String text && !text.isBlank()) {
      target.accept(text);
    }
  }
}
