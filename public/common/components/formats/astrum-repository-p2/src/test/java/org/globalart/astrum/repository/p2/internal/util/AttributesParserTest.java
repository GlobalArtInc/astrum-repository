package org.globalart.astrum.repository.p2.internal.util;

import java.io.File;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.jar.Attributes;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.jar.JarInputStream;
import java.util.jar.JarOutputStream;
import java.util.jar.Manifest;
import java.util.zip.GZIPOutputStream;

import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.apache.commons.compress.java.util.jar.Pack200;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class AttributesParserTest
{
  private static final String FEATURE_JAR = "org.eclipse.core.runtime.feature_1.2.100.v20170912-1859.jar";

  private static final String BUNDLE_JAR = "org.tigris.subversion.clientadapter.svnkit_1.7.5.jar";

  private static final String SOURCE_JAR = "org.eclipse.e4.tools.emf.editor3x.source_4.7.0.v20170712-1432.jar";

  private static final String PACK_GZ = "com.google.code.atinject.tck_1.1.0.v20160901-1938.jar.pack.gz";

  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private TempBlobConverter converter;

  private AttributesParserFeatureXml featureXmlParser;

  private AttributesParserManifest manifestParser;

  private P2TempBlobUtils tempBlobUtils;

  @Before
  public void setUp() throws Exception {
    converter = new TempBlobConverter();
    PropertyParser propertyParser = new PropertyParser(converter);
    featureXmlParser = new AttributesParserFeatureXml(converter, propertyParser);
    manifestParser = new AttributesParserManifest(converter, propertyParser);
    tempBlobUtils = new P2TempBlobUtils(featureXmlParser, manifestParser);
  }

  @Test
  public void readsFeatureXml() throws Exception {
    P2Attributes attributes = featureXmlParser.getAttributesFromBlob(blob(FEATURE_JAR), "jar");
    assertEquals("org.eclipse.core.runtime.feature", attributes.getComponentName());
    assertEquals("1.2.100.v20170912-1859", attributes.getComponentVersion());
    assertEquals("Eclipse Core Runtime Infrastructure", attributes.getPluginName());
  }

  @Test
  public void readsBundleManifests() throws Exception {
    P2Attributes attributes = manifestParser.getAttributesFromBlob(blob(BUNDLE_JAR), "jar");
    assertEquals("org.tigris.subversion.clientadapter.svnkit", attributes.getComponentName());
    assertEquals("1.7.5", attributes.getComponentVersion());
    assertEquals("SVNKit Client Adapter", attributes.getPluginName());

    attributes = manifestParser.getAttributesFromBlob(blob(SOURCE_JAR), "jar");
    assertEquals("org.eclipse.e4.tools.emf.editor3x.source", attributes.getComponentName());
    assertEquals("4.7.0.v20170712-1432", attributes.getComponentVersion());
    assertEquals("Editor3x Source", attributes.getPluginName());

    assertTrue(featureXmlParser.getAttributesFromBlob(blob(BUNDLE_JAR), "jar").isEmpty());
  }

  @Test
  public void unpacksPack200Archives() throws Exception {
    Path packGz = temporaryFolder.newFile("svnkit.jar.pack.gz").toPath();
    try (JarFile jar = new JarFile(new File(getClass().getResource(BUNDLE_JAR).toURI()));
         OutputStream out = new GZIPOutputStream(Files.newOutputStream(packGz))) {
      Pack200.newPacker().pack(jar, out);
    }
    TempBlob blob = mock(TempBlob.class);
    when(blob.get()).thenAnswer(invocation -> Files.newInputStream(packGz));

    int classes = 0;
    try (InputStream in = converter.getJarFromPackGz(blob); JarInputStream jar = new JarInputStream(in)) {
      JarEntry entry;
      while ((entry = jar.getNextJarEntry()) != null) {
        if (entry.getName().endsWith(".class")) {
          classes++;
        }
      }
    }
    assertTrue(classes > 0);

    P2Attributes attributes = manifestParser.getAttributesFromBlob(blob, "jar.pack.gz");
    assertEquals("org.tigris.subversion.clientadapter.svnkit", attributes.getComponentName());
    assertEquals("1.7.5", attributes.getComponentVersion());
  }

  @Test
  public void keepsPathAttributesOfUnsupportedPack200Versions() {
    P2Attributes fromPath = P2Attributes.builder()
        .componentName("com.google.code.atinject.tck")
        .componentVersion("1.1.0.v20160901-1938")
        .extension("jar.pack.gz")
        .path("/plugins/" + PACK_GZ)
        .build();
    assertEquals(fromPath, tempBlobUtils.mergeAttributesFromTempBlob(blob(PACK_GZ), fromPath));
  }

  @Test
  public void readsManifestsWrittenFirstAndResolvesLocalization() throws Exception {
    Path jar = bundle("org.example.localized", "%bundleName", "bundleName=Localized Bundle\n");
    P2Attributes attributes = manifestParser.getAttributesFromBlob(blob(jar), "jar");
    assertEquals("org.example.localized", attributes.getComponentName());
    assertEquals("1.2.3.qualifier", attributes.getComponentVersion());
    assertEquals("Localized Bundle", attributes.getPluginName());

    jar = bundle("org.example.unresolved", "%missing", null);
    attributes = manifestParser.getAttributesFromBlob(blob(jar), "jar");
    assertEquals("org.example.unresolved", attributes.getComponentName());
    assertNull(attributes.getPluginName());
  }

  @Test
  public void mergesJarAttributesOverPathAttributes() {
    P2Attributes fromPath = P2Attributes.builder()
        .componentName("org.tigris.subversion.clientadapter.svnkit")
        .componentVersion("1.7.5")
        .extension("jar")
        .path("/plugins/" + BUNDLE_JAR)
        .build();
    P2Attributes merged = tempBlobUtils.mergeAttributesFromTempBlob(blob(BUNDLE_JAR), fromPath);
    assertEquals("SVNKit Client Adapter", merged.getPluginName());
    assertEquals("/plugins/" + BUNDLE_JAR, merged.getPath());
    assertEquals("jar", merged.getExtension());
  }

  @Test
  public void keepsPathAttributesWhenTheJarCannotBeRead() {
    P2Attributes fromPath = P2Attributes.builder()
        .componentName("broken")
        .componentVersion("1.0.0")
        .extension("jar.pack.gz")
        .path("/plugins/broken_1.0.0.jar.pack.gz")
        .build();
    assertEquals(fromPath, tempBlobUtils.mergeAttributesFromTempBlob(blob(BUNDLE_JAR), fromPath));
  }

  private Path bundle(final String symbolicName, final String bundleName, final String properties) throws Exception {
    Manifest manifest = new Manifest();
    manifest.getMainAttributes().put(Attributes.Name.MANIFEST_VERSION, "1.0");
    manifest.getMainAttributes().putValue("Bundle-SymbolicName", symbolicName + ";singleton:=true");
    manifest.getMainAttributes().putValue("Bundle-Version", "1.2.3.qualifier");
    manifest.getMainAttributes().putValue("Bundle-Name", bundleName);
    Path jar = temporaryFolder.newFile(symbolicName + ".jar").toPath();
    try (JarOutputStream out = new JarOutputStream(Files.newOutputStream(jar), manifest)) {
      if (properties != null) {
        out.putNextEntry(new JarEntry("OSGI-INF/l10n/bundle.properties"));
        out.write(properties.getBytes(StandardCharsets.ISO_8859_1));
        out.closeEntry();
      }
      out.putNextEntry(new JarEntry("org/example/Empty.class"));
      out.closeEntry();
    }
    return jar;
  }

  private TempBlob blob(final Path path) {
    TempBlob blob = mock(TempBlob.class);
    when(blob.get()).thenAnswer(invocation -> Files.newInputStream(path));
    return blob;
  }

  private TempBlob blob(final String name) {
    TempBlob blob = mock(TempBlob.class);
    when(blob.get()).thenAnswer(invocation -> getClass().getResourceAsStream(name));
    return blob;
  }
}
