package org.globalart.astrum.repository.pypi.internal.search;

import java.util.function.Consumer;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_CLASSIFIERS;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_DESCRIPTION;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_KEYWORDS;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_SUMMARY;

@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    NestedAttributesMap attributes = asset.attributes(PyPiFormat.NAME);
    add(attributes, P_CLASSIFIERS, searchRecord::addFormatFieldValue1);
    add(attributes, P_DESCRIPTION, searchRecord::addFormatFieldValue2);
    add(attributes, P_KEYWORDS, searchRecord::addFormatFieldValue3);
    add(attributes, P_SUMMARY, searchRecord::addFormatFieldValue4);
  }

  private static void add(final NestedAttributesMap attributes, final String key, final Consumer<String> target) {
    Object value = attributes.get(key);
    if (value instanceof String text && !text.isBlank()) {
      target.accept(text);
    }
  }
}
