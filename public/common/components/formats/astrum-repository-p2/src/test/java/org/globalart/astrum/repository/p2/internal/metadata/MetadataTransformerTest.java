package org.globalart.astrum.repository.p2.internal.metadata;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

import org.globalart.astrum.repository.p2.internal.proxy.StreamCopier;

import org.apache.commons.compress.compressors.xz.XZCompressorInputStream;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.not;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class MetadataTransformerTest
{
  private static final String BASE = "https://download.eclipse.org/releases/2019-09/";

  @Test
  public void removesMirrorsFromPlainArtifacts() throws IOException {
    RemoveMirrorTransformer transformer = new RemoveMirrorTransformer();
    String xml = new String(transform("/artifacts.xml", "artifacts.xml", transformer), StandardCharsets.UTF_8);
    assertEquals("mirrors.xml", transformer.getMirrorsUrl().orElseThrow());
    assertThat(xml, not(containsString("p2.mirrorsURL")));
    assertThat(xml, containsString("<properties size=\"4\">"));
    assertThat(xml, containsString("<?artifactRepository"));
    assertThat(xml, containsString("publishPackFilesAsSiblings"));
    assertThat(xml, containsString("${repoUrl}/plugins/${id}_${version}.jar.pack.gz"));
  }

  @Test
  public void removesMirrorsFromJarAndXz() throws IOException {
    RemoveMirrorTransformer transformer = new RemoveMirrorTransformer();
    String xml = unzip(transform("/artifacts.jar", "artifacts.xml", transformer), "artifacts.xml");
    assertTrue(transformer.getMirrorsUrl().isPresent());
    assertThat(xml, not(containsString("p2.mirrorsURL")));

    transformer = new RemoveMirrorTransformer();
    byte[] xz = transform("/artifacts.xml.xz", "artifacts.xml", transformer);
    try (InputStream in = new XZCompressorInputStream(new ByteArrayInputStream(xz))) {
      xml = new String(in.readAllBytes(), StandardCharsets.UTF_8);
    }
    assertTrue(transformer.getMirrorsUrl().isPresent());
    assertThat(xml, not(containsString("p2.mirrorsURL")));
    assertThat(xml, containsString("<artifacts"));
  }

  @Test
  public void keepsArtifactsWithoutMirrors() throws IOException {
    RemoveMirrorTransformer transformer = new RemoveMirrorTransformer();
    String xml = new String(transform("/artifacts-without-absolute.xml", "artifacts.xml", transformer),
        StandardCharsets.UTF_8);
    assertTrue(transformer.getMirrorsUrl().isEmpty());
    assertThat(xml, containsString("<properties size=\"4\">"));
  }

  @Test
  public void rewritesRootCompositeChildrenToSiteHashes() throws IOException {
    CompositeRepositoryRewriter rewriter = new CompositeRepositoryRewriter(URI.create(BASE), true);
    String xml = new String(transform("/compositeArtifacts.xml", "compositeArtifacts.xml", rewriter),
        StandardCharsets.UTF_8);

    List<String> urls = rewriter.getUrls();
    assertEquals(List.of(
        "https://download.eclipse.org/technology/epp/packages/2019-09/",
        "https://download.eclipse.org/releases/2019-09/201909181001/"), urls);
    for (String url : urls) {
      assertThat(xml, containsString("location=\"" + UriToSiteHashUtil.map(url) + "/\""));
    }
    assertThat(xml, not(containsString("technology/epp")));
    assertThat(xml, containsString("<?compositeArtifactRepository"));
  }

  @Test
  public void rewritesNestedCompositeChildrenRelativeToTheParent() throws IOException {
    CompositeRepositoryRewriter rewriter = new CompositeRepositoryRewriter(URI.create(BASE), false);
    String xml = unzip(transform("/compositeContent.jar", "compositeContent.xml", rewriter), "compositeContent.xml");
    for (String url : rewriter.getUrls()) {
      assertThat(xml, containsString("location=\"../" + UriToSiteHashUtil.map(url) + "/\""));
    }
    assertThat(xml, containsString("<?compositeMetadataRepository"));
  }

  @Test
  public void rewritesAbsoluteChildren() throws IOException {
    String composite = "<?xml version='1.0' encoding='UTF-8'?>\n" +
        "<?compositeMetadataRepository version='1.0.0'?>\n" +
        "<repository name='r' type='org.eclipse.equinox.internal.p2.metadata.repository.CompositeMetadataRepository'" +
        " version='1.0.0'>\n" +
        "  <children size='1'>\n" +
        "    <child location='https://download.eclipse.org/eclipse/updates/4.31'/>\n" +
        "  </children>\n" +
        "</repository>\n";
    CompositeRepositoryRewriter rewriter = new CompositeRepositoryRewriter(URI.create(BASE), true);
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    StreamCopier.copierFor("/compositeContent.xml", "compositeContent.xml",
        new ByteArrayInputStream(composite.getBytes(StandardCharsets.UTF_8)), out).process(rewriter);
    assertEquals(List.of("https://download.eclipse.org/eclipse/updates/4.31/"), rewriter.getUrls());
    assertThat(out.toString(StandardCharsets.UTF_8),
        containsString("location=\"" + UriToSiteHashUtil.map("https://download.eclipse.org/eclipse/updates/4.31") +
            "/\""));
  }

  private byte[] transform(final String resource, final String entry, final XmlTransformer transformer)
      throws IOException
  {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (InputStream in = getClass().getResourceAsStream(resource.substring(1))) {
      StreamCopier.copierFor(resource, entry, in, out).process(transformer);
    }
    return out.toByteArray();
  }

  private static String unzip(final byte[] bytes, final String entry) throws IOException {
    try (ZipInputStream zip = new ZipInputStream(new ByteArrayInputStream(bytes))) {
      ZipEntry next = zip.getNextEntry();
      assertEquals(entry, next.getName());
      return new String(zip.readAllBytes(), StandardCharsets.UTF_8);
    }
  }
}
