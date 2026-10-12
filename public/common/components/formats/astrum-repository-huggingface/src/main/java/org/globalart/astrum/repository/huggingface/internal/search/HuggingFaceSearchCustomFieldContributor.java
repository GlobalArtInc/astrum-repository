package org.globalart.astrum.repository.huggingface.internal.search;

import java.util.Collection;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.search.sql.SearchCustomFieldContributor;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.ATTRIBUTES;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.FILE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.REVISIONS;

/**
 * Indexes the branches and tags a commit was seen under, also as keywords so that searching for {@code v1.0} finds
 * the commit, and the path of the file.
 */
@Component
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceSearchCustomFieldContributor
    implements SearchCustomFieldContributor
{
  @Override
  public void populateSearchCustomFields(final SearchRecord searchRecord, final Asset asset) {
    Object revisions = asset.attributes(ATTRIBUTES).get(REVISIONS);
    if (revisions instanceof Collection<?> values) {
      values.stream()
          .map(String::valueOf)
          .filter(value -> !value.isBlank())
          .forEach(value -> {
            searchRecord.addFormatFieldValue1(value, true);
            searchRecord.addKeyword(value);
          });
    }
    Object file = asset.attributes(ATTRIBUTES).get(FILE);
    if (file instanceof String path && !path.isBlank()) {
      searchRecord.addFormatFieldValue2(path);
    }
  }
}
