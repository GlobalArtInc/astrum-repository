
package org.globalart.astrum.rapture.internal;

import java.util.Arrays;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEdition;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class BundleStateContributorTest
{
  @Mock
  private AstrumEditionSelector astrumEditionSelector;

  @Mock
  private AstrumEdition astrumEdition;

  private BundleStateContributor bundleStateContributor;

  @Before
  public void setUp() {
    bundleStateContributor = new BundleStateContributor(astrumEditionSelector);
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
  }

  @Test
  public void testGetState_filtersExternalDependencies() {
    // Mock modules list including both internal and external dependencies
    List<String> allModules = Arrays.asList(
        "astrum-core-1.0.jar",
        "astrum-rapture-1.0.jar",
        "globalart-astrum-repository-1.0.jar",
        "vendor-plugin-1.0.jar",
        "spring-boot-3.3.6.jar",
        "guava-32.1.3.jar",
        "jackson-databind-2.17.0.jar",
        "astrum-test-module-2.0.jar");
    when(astrumEdition.getModules()).thenReturn(allModules);

    Map<String, Object> state = bundleStateContributor.getState();
    @SuppressWarnings("unchecked")
    List<String> modules = (List<String>) state.get(BundleStateContributor.STATE_ID);

    // Only internal Astrum modules with "astrum-" prefix should be returned
    assertThat(modules)
        .hasSize(3)
        .containsExactlyInAnyOrder(
            "astrum-core-1.0.jar",
            "astrum-rapture-1.0.jar",
            "astrum-test-module-2.0.jar");
  }

  @Test
  public void testGetState_cachesModuleList() {
    List<String> modulesList = Arrays.asList("astrum-core-1.0.jar", "astrum-rapture-1.0.jar");
    when(astrumEdition.getModules()).thenReturn(modulesList);

    // Call getState twice
    bundleStateContributor.getState();
    Map<String, Object> state2 = bundleStateContributor.getState();

    @SuppressWarnings("unchecked")
    List<String> modules = (List<String>) state2.get(BundleStateContributor.STATE_ID);

    assertThat(modules).hasSize(2);
    // Verify that getModules() was only called once (cached)
    verify(astrumEdition, times(1)).getModules();
  }

  @Test
  public void testGetState_handlesEmptyModuleList() {
    when(astrumEdition.getModules()).thenReturn(List.of());

    Map<String, Object> state = bundleStateContributor.getState();
    @SuppressWarnings("unchecked")
    List<String> modules = (List<String>) state.get(BundleStateContributor.STATE_ID);

    assertThat(modules).isEmpty();
  }

  @Test
  public void testGetState_handlesOnlyExternalDependencies() {
    // All external dependencies, no internal modules
    List<String> allModules = Arrays.asList(
        "spring-boot-3.3.6.jar",
        "guava-32.1.3.jar",
        "jackson-databind-2.17.0.jar");
    when(astrumEdition.getModules()).thenReturn(allModules);

    Map<String, Object> state = bundleStateContributor.getState();
    @SuppressWarnings("unchecked")
    List<String> modules = (List<String>) state.get(BundleStateContributor.STATE_ID);

    assertThat(modules).isEmpty();
  }

  @Test
  public void testGetState_includesOnlyAstrumPrefix() {
    // Test that only "astrum-" prefix modules are included
    List<String> allModules = Arrays.asList(
        "astrum-base-1.0.jar",
        "globalart-astrum-pro-2.0.jar",
        "vendor-cloud-3.0.jar",
        "other-library-1.0.jar");
    when(astrumEdition.getModules()).thenReturn(allModules);

    Map<String, Object> state = bundleStateContributor.getState();
    @SuppressWarnings("unchecked")
    List<String> modules = (List<String>) state.get(BundleStateContributor.STATE_ID);

    assertThat(modules)
        .hasSize(1)
        .containsExactly("astrum-base-1.0.jar");
  }

  @Test
  public void testGetState_returnsCorrectStateId() {
    when(astrumEdition.getModules()).thenReturn(List.of());

    Map<String, Object> state = bundleStateContributor.getState();

    assertThat(state)
        .containsKey(BundleStateContributor.STATE_ID)
        .containsKey("activeBundles");
  }
}
