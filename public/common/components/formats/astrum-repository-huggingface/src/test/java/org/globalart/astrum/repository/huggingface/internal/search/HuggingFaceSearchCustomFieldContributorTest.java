package org.globalart.astrum.repository.huggingface.internal.search;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.junit.jupiter.api.Test;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

class HuggingFaceSearchCustomFieldContributorTest
{
  private final HuggingFaceSearchCustomFieldContributor underTest = new HuggingFaceSearchCustomFieldContributor();

  @Test
  void indexesRevisionsAndTheFilePath() {
    Map<String, Object> huggingface = new HashMap<>();
    huggingface.put("revisions", List.of("main", "v1.0"));
    huggingface.put("file", "onnx/model.onnx");
    SearchRecord record = mock(SearchRecord.class);

    underTest.populateSearchCustomFields(record, asset(huggingface));

    verify(record).addFormatFieldValue1("main", true);
    verify(record).addFormatFieldValue1("v1.0", true);
    verify(record).addKeyword("main");
    verify(record).addKeyword("v1.0");
    verify(record).addFormatFieldValue2("onnx/model.onnx");
  }

  @Test
  void ignoresAssetsWithoutAttributes() {
    SearchRecord record = mock(SearchRecord.class);
    underTest.populateSearchCustomFields(record, asset(new HashMap<>()));
    verifyNoInteractions(record);
  }

  private static Asset asset(final Map<String, Object> huggingface) {
    Map<String, Object> backing = new HashMap<>();
    backing.put("huggingface", huggingface);
    Asset asset = mock(Asset.class);
    when(asset.attributes()).thenReturn(new NestedAttributesMap("attributes", backing));
    when(asset.attributes("huggingface")).thenCallRealMethod();
    return asset;
  }
}
