
package org.globalart.astrum.content.maven.internal.search.table;

import java.util.Optional;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mockito;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MavenSearchCustomFieldContributorTest

{
  @Mock
  private Component component;

  private NestedAttributesMap attributes;

  private Asset asset;

  private NestedAttributesMap childAttributes;

  private MavenSearchCustomFieldContributor underTest;

  @Before
  public void setup() {
    underTest = new MavenSearchCustomFieldContributor();

    asset = Mockito.mock(Asset.class);

    attributes = new NestedAttributesMap();
    childAttributes = attributes.child(Maven2Format.NAME);
  }

  @Test
  public void customSearchFieldsAreSetCorrectly() {
    mockAttributes();
    mockComponentAttributes();

    SearchRecord searchRecord = mock(SearchRecord.class);

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addFormatFieldValue1("1.1.1", true);
    verify(searchRecord).addFormatFieldValue2("testExtension");
    verify(searchRecord).addFormatFieldValue3("testClassifier");

    childAttributes.set("baseVersion", "1.1-SNAPSHOT");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addFormatFieldValue1("1.1-SNAPSHOT", true);
  }

  @Test
  public void operationIsSuccessfulWithNoCustomSearchData() {
    SearchRecord searchRecord = mock(SearchRecord.class);

    when(asset.attributes()).thenReturn(attributes);

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord, never()).addFormatFieldValue1(any());
    verify(searchRecord, never()).addFormatFieldValue2(any());
  }

  private void mockAttributes() {
    childAttributes.set("baseVersion", "1.1.1");
    childAttributes.set("extension", "testExtension");
    childAttributes.set("classifier", "testClassifier");
    when(asset.attributes()).thenReturn(attributes);
  }

  private void mockComponentAttributes() {
    when(asset.component()).thenReturn(Optional.of(component));
    when(component.attributes()).thenReturn(attributes);
  }
}
