
package org.globalart.astrum.repository.apt.datastore.internal.search.sql;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class AptSearchCustomFieldContributorTest
{
  @Mock
  private Asset asset;

  private final AptSearchCustomFieldContributor underTest = new AptSearchCustomFieldContributor();

  @Test
  public void shouldAddPathWithoutLeadingSlash() {
    SearchRecord data = mock(SearchRecord.class);
    String path = "/org/foo/1.0/foo-1.0.txt";
    when(asset.path()).thenReturn(path);

    underTest.populateSearchCustomFields(data, asset);

    verify(data).addKeyword(path.substring(1));
  }
}
