
package org.globalart.astrum.content.raw.internal.search;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.search.sql.SearchRecord;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class RawSearchCustomFieldContributorTest
{
  @Mock
  private SearchRecord searchRecord;

  @Mock
  private Asset asset;

  private RawSearchCustomFieldContributor underTest;

  @BeforeEach
  void setUp() {
    underTest = new RawSearchCustomFieldContributor();
  }

  @Test
  void registersBasenameAsComponentNameAliasForNestedPath() {
    when(asset.path()).thenReturn("/foo/bar/test.txt");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addAliasComponentName("test.txt");
  }

  @Test
  void registersBasenameAsComponentNameAliasForDeepPath() {
    when(asset.path()).thenReturn("/a/b/c/d/e/file.name.with.dots.tar.gz");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addAliasComponentName("file.name.with.dots.tar.gz");
  }

  @Test
  void registersBasenameAsComponentNameAliasForSingleSlashPath() {
    when(asset.path()).thenReturn("/test.txt");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addAliasComponentName("test.txt");
  }

  @Test
  void registersPathItselfWhenNoSlashPresent() {
    when(asset.path()).thenReturn("plain.txt");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addAliasComponentName("plain.txt");
  }

  @Test
  void skipsWhenPathIsNull() {
    when(asset.path()).thenReturn(null);

    underTest.populateSearchCustomFields(searchRecord, asset);

    verifyNoInteractions(searchRecord);
  }

  @Test
  void skipsWhenPathIsEmpty() {
    when(asset.path()).thenReturn("");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verifyNoInteractions(searchRecord);
  }

  @Test
  void skipsWhenPathEndsWithSlash() {
    when(asset.path()).thenReturn("/foo/bar/");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord, never()).addAliasComponentName(anyString());
  }

  @Test
  void skipsWhenPathIsSingleSlash() {
    when(asset.path()).thenReturn("/");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord, never()).addAliasComponentName(anyString());
  }

  @Test
  void registersBasenamePreservingDots() {
    when(asset.path()).thenReturn("/path/to/foo.txt");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addAliasComponentName("foo.txt");
  }

  @Test
  void registersBasenameWithoutExtension() {
    when(asset.path()).thenReturn("/path/to/README");

    underTest.populateSearchCustomFields(searchRecord, asset);

    verify(searchRecord).addAliasComponentName("README");
  }
}
