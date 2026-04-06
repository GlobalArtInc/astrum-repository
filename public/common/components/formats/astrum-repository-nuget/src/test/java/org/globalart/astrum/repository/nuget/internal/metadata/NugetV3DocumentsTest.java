package org.globalart.astrum.repository.nuget.internal.metadata;

import java.io.IOException;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.Dependency;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata.DependencyGroup;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class NugetV3DocumentsTest
{
  private static final String REPOSITORY = "http://localhost/repository/nuget-hosted";

  private static NugetPackageMetadata metadata(final String id, final String version, final String tags) {
    NugetPackageMetadata metadata = new NugetPackageMetadata();
    metadata.setId(id);
    metadata.setVersion(NugetVersion.parse(version).orElseThrow());
    metadata.setText("tags", tags);
    metadata.setText("description", id + " description");
    metadata.setDependencyGroups(List.of(new DependencyGroup("net8.0", List.of(new Dependency("Dep", "1.0")))));
    return metadata;
  }

  @Test
  public void normalizesRanges() {
    assertEquals("[1.0.0, )", NugetV3Documents.normalizeRange("1.0"));
    assertEquals("[1.0.0, 2.0.0)", NugetV3Documents.normalizeRange("[1.0,2.0)"));
    assertEquals("[1.0.0, 1.0.0]", NugetV3Documents.normalizeRange("[1.0]"));
    assertEquals("(, 3.0.0]", NugetV3Documents.normalizeRange("(,3.0]"));
    assertEquals("(, )", NugetV3Documents.normalizeRange(""));
  }

  @Test
  @SuppressWarnings("unchecked")
  public void buildsRegistrationIndex() throws IOException {
    String json = NugetV3Documents.registrationIndex(REPOSITORY, "Foo.Bar",
        List.of(metadata("Foo.Bar", "2.0.0", ""), metadata("Foo.Bar", "1.0.0-Beta", "")));
    Map<String, Object> index = NugetV3Documents.read(json);
    Map<String, Object> page = ((List<Map<String, Object>>) index.get("items")).get(0);
    assertEquals("1.0.0-beta", page.get("lower"));
    assertEquals("2.0.0", page.get("upper"));
    List<Map<String, Object>> leaves = (List<Map<String, Object>>) page.get("items");
    Map<String, Object> entry = (Map<String, Object>) leaves.get(0).get("catalogEntry");
    assertEquals("1.0.0-Beta", entry.get("version"));
    assertEquals(REPOSITORY + "/v3/content/foo.bar/1.0.0-beta/foo.bar.1.0.0-beta.nupkg", entry.get("packageContent"));
    Map<String, Object> group = ((List<Map<String, Object>>) entry.get("dependencyGroups")).get(0);
    Map<String, Object> dependency = ((List<Map<String, Object>>) group.get("dependencies")).get(0);
    assertEquals("[1.0.0, )", dependency.get("range"));
    assertEquals(REPOSITORY + "/v3/registration/dep/index.json", dependency.get("registration"));
  }

  @Test
  public void searchesByTermsAndFilters() {
    List<NugetPackageMetadata> packages = List.of(
        metadata("Foo.Bar", "1.0.0", "json tools"),
        metadata("Foo.Bar", "2.0.0-beta.1", "json tools"),
        metadata("Other", "1.0.0", "xml"));
    NugetSearch.Result stable = new NugetSearch("json", 0, 20, false, false).search(packages);
    assertEquals(1, stable.totalHits());
    assertEquals("1.0.0", stable.hits().get(0).latest().getVersion().toNormalizedString());
    NugetSearch.Result prerelease = new NugetSearch("foo", 0, 20, true, true).search(packages);
    assertEquals("2.0.0-beta.1", prerelease.hits().get(0).latest().getVersion().toNormalizedString());
    NugetSearch.Result hidden = new NugetSearch("foo", 0, 20, true, false).search(packages);
    assertEquals("1.0.0", hidden.hits().get(0).latest().getVersion().toNormalizedString());
    assertEquals(1, new NugetSearch("packageid:other", 0, 20, false, false).search(packages).totalHits());
    assertEquals(2, new NugetSearch(null, 0, 20, false, false).search(packages).totalHits());
    assertTrue(new NugetSearch(null, 5, 20, false, false).search(packages).hits().isEmpty());
  }
}
