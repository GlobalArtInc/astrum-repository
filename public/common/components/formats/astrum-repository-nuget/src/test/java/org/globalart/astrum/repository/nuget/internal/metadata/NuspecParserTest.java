package org.globalart.astrum.repository.nuget.internal.metadata;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

public class NuspecParserTest
{
  private static final String NUSPEC = """
      <?xml version="1.0" encoding="utf-8"?>
      <package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">
        <metadata minClientVersion="2.12">
          <id>Contoso.Utility</id>
          <version>1.2.0-beta.3+sha.abc</version>
          <title>Contoso Utility</title>
          <authors>Alice, Bob</authors>
          <description>Utility &amp; helpers</description>
          <tags>utility helpers</tags>
          <license type="expression">MIT</license>
          <requireLicenseAcceptance>true</requireLicenseAcceptance>
          <dependencies>
            <group targetFramework="net8.0">
              <dependency id="Newtonsoft.Json" version="13.0.1" />
            </group>
            <group targetFramework="netstandard2.0" />
          </dependencies>
        </metadata>
      </package>
      """;

  private static byte[] nupkg(final String nuspecName, final String nuspec) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZipOutputStream zip = new ZipOutputStream(out)) {
      zip.putNextEntry(new ZipEntry("[Content_Types].xml"));
      zip.write("<Types/>".getBytes(StandardCharsets.UTF_8));
      zip.putNextEntry(new ZipEntry("lib/net8.0/Contoso.Utility.dll"));
      zip.write(new byte[]{1, 2, 3});
      zip.putNextEntry(new ZipEntry(nuspecName));
      zip.write(nuspec.getBytes(StandardCharsets.UTF_8));
    }
    return out.toByteArray();
  }

  @Test
  public void parsesMetadataFromPackage() throws IOException {
    NugetPackageMetadata metadata =
        NuspecParser.parsePackage(new ByteArrayInputStream(nupkg("Contoso.Utility.nuspec", NUSPEC)));
    assertEquals("Contoso.Utility", metadata.getId());
    assertEquals("1.2.0-beta.3+sha.abc", metadata.getVersion().toFullString());
    assertEquals("Utility & helpers", metadata.text("description"));
    assertEquals(List.of("Alice", "Bob"), metadata.getAuthorList());
    assertEquals(List.of("utility", "helpers"), metadata.getTagList());
    assertEquals("MIT", metadata.text("license_expression"));
    assertEquals("2.12", metadata.text("min_client_version"));
    assertTrue(metadata.isRequireLicenseAcceptance());
    assertTrue(metadata.isSemVer2());
    assertEquals(2, metadata.getDependencyGroups().size());
    assertEquals("net8.0", metadata.getDependencyGroups().get(0).getTargetFramework());
    assertEquals("Newtonsoft.Json", metadata.getDependencyGroups().get(0).getDependencies().get(0).getId());
    assertTrue(metadata.getDependencyGroups().get(1).getDependencies().isEmpty());
  }

  @Test
  public void roundTripsThroughAttributes() throws IOException {
    NugetPackageMetadata metadata =
        NuspecParser.parsePackage(new ByteArrayInputStream(nupkg("Contoso.Utility.nuspec", NUSPEC)));
    Map<String, Object> attributes = metadata.toAttributes();
    NugetPackageMetadata copy = NugetPackageMetadata.fromAttributes(attributes);
    assertEquals(metadata.getId(), copy.getId());
    assertEquals(metadata.getVersion(), copy.getVersion());
    assertEquals(metadata.getTagList(), copy.getTagList());
    assertEquals("13.0.1", copy.getDependencyGroups().get(0).getDependencies().get(0).getRange());
    assertNull(NugetPackageMetadata.fromAttributes(Map.of("id", "x")));
  }

  @Test
  public void ignoresNuspecFilesInSubdirectories() throws IOException {
    assertFalse(NuspecParser.readNuspec(new ByteArrayInputStream(nupkg("content/other.nuspec", NUSPEC))).isPresent());
  }

  @Test(expected = InvalidNugetPackageException.class)
  public void rejectsInvalidId() {
    NuspecParser.parseNuspec(NUSPEC.replace("Contoso.Utility", "bad id").getBytes(StandardCharsets.UTF_8));
  }

  @Test(expected = InvalidNugetPackageException.class)
  public void rejectsDoctype() {
    String withDoctype = NUSPEC.replace("<package", "<!DOCTYPE package [<!ENTITY x SYSTEM \"file:///etc/passwd\">]><package");
    NuspecParser.parseNuspec(withDoctype.getBytes(StandardCharsets.UTF_8));
  }
}
