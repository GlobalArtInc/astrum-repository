package org.globalart.astrum.repository.terraform.internal.search;

import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.junit.Test;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class TerraformSearchCustomFieldContributorTest
{
  @Test
  public void indexesTheAddressOfAModule() {
    Asset asset = mock(Asset.class);
    when(asset.attributes("terraform")).thenReturn(new NestedAttributesMap("terraform", Map.of(
        "kind", "module", "namespace", "acme", "name", "vpc", "provider", "aws")));
    SearchRecord record = mock(SearchRecord.class);

    new TerraformSearchCustomFieldContributor().populateSearchCustomFields(record, asset);

    verify(record).addFormatFieldValue1("acme");
    verify(record).addFormatFieldValue2("vpc");
    verify(record).addFormatFieldValue3("aws");
    verify(record).addFormatFieldValue4("module");
    verify(record).addKeyword("vpc");
    verify(record, never()).addKeyword("module");
  }
}
