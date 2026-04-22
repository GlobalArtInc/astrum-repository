
package org.globalart.astrum.repository.composer.internal.hosted;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.ComposerContentFacet;
import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.fluent.internal.FluentComponentQueryImpl;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import java.util.Map;
import java.util.Optional;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.*;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerHostedFacetImplTest
{
  private static final String VENDOR = "vendor";

  private static final String PROJECT = "project";

  private static final String VERSION = "version";

  private static final String SRC_TYPE = "src-type";

  private static final String SRC_URL = "src-url";

  private static final String SRC_REF = "src-ref";

  private static final String ZIPBALL_PATH = "/vendor/project/version/vendor-project-version.zip";

  private static final String PROVIDER_PATH = "/p/vendor/project.json";

  @Mock
  private Repository repository;

  @Mock
  private ComposerContentFacet composerContentFacet;

  @Mock
  private ComposerJsonProcessor composerJsonProcessor;

  @Mock
  private Payload payload;

  @Mock
  private Content content;

  @Mock
  private FluentComponents components;

  private ComposerHostedFacetImpl underTest;

  @Before
  public void setUp() throws Exception {
    when(repository.facet(ComposerContentFacet.class)).thenReturn(composerContentFacet);
    when(composerContentFacet.components()).thenReturn(components);

    underTest = spy(new ComposerHostedFacetImpl(composerJsonProcessor));
    underTest.attach(repository);
  }

  @Test
  public void testUpload() throws Exception {
    underTest.upload(VENDOR, PROJECT, VERSION, SRC_TYPE, SRC_URL, SRC_REF, payload);
    verify(composerContentFacet).put(ZIPBALL_PATH, payload, SRC_TYPE, SRC_URL, SRC_REF);
  }

  @Test
  public void testGetZipball() throws Exception {
    when(composerContentFacet.get(ZIPBALL_PATH)).thenReturn(Optional.of(content));
    assertThat(underTest.getZipball(ZIPBALL_PATH), is(content));
  }

  @Test
  public void testGetPackagesJson() throws Exception {
    when(composerJsonProcessor.generatePackagesFromComponents(repository, components)).thenReturn(content);
    assertThat(underTest.getPackagesJson(), is(content));
  }

  @Test
  public void testGetListJson() throws Exception {
    // Without filter
    when(composerJsonProcessor.generateListFromComponents(components)).thenReturn(content);
    assertThat(underTest.getListJson(null), is(content));

    // With filter
    FluentQuery<FluentComponent> query = mock(FluentComponentQueryImpl.class);
    when(components.byFilter("namespace LIKE #{filterParams.vendor} AND name LIKE #{filterParams.project}",
        ImmutableMap.of("vendor", "test", "project", "%"))).thenReturn(query);
    when(composerJsonProcessor.generateListFromComponents(query)).thenReturn(content);
    assertThat(underTest.getListJson("test/*"), is(content));

    when(components.byFilter("namespace LIKE #{filterParams.vendor} AND name LIKE #{filterParams.project}",
        ImmutableMap.of("vendor", "%abc%", "project", "pr0_j3cT"))).thenReturn(query);
    when(composerJsonProcessor.generateListFromComponents(query)).thenReturn(content);
    assertThat(underTest.getListJson("*abc**/pr0_j3cT"), is(content));

    // Invalid filter
    when(composerJsonProcessor.generateListFromComponents(null)).thenReturn(content);
    assertThat(underTest.getListJson("In\\al1d"), is(content));
  }

  @Test
  public void testGetProviderJson() throws Exception {
    when(composerContentFacet.get(PROVIDER_PATH)).thenReturn(Optional.of(content));
    assertThat(underTest.getProviderJson(VENDOR, PROJECT), is(content));
  }

  @Test
  public void testBuildProviderJson() throws Exception {
    ArgumentCaptor<String> filter = ArgumentCaptor.forClass(String.class);
    ArgumentCaptor<Map<String, Object>> filterArgs = ArgumentCaptor.forClass(Map.class);

    FluentQuery<FluentComponent> query = mock(FluentQuery.class);
    when(components.byFilter(filter.capture(), filterArgs.capture())).thenReturn(query);
    when(composerJsonProcessor.buildProviderJson(repository, composerContentFacet, query))
        .thenReturn(Optional.of(content));

    Optional<Content> res = underTest.rebuildProviderJson(VENDOR, PROJECT);
    assertThat(res.isPresent(), is(true));
    assertThat(res.get(), is(content));
    assertThat(filter.getValue(), is("namespace = #{filterParams.vendor} AND name = #{filterParams.project}"));
    assertThat(filterArgs.getValue(), is(ImmutableMap.of("vendor", VENDOR, "project", PROJECT)));
  }
}
