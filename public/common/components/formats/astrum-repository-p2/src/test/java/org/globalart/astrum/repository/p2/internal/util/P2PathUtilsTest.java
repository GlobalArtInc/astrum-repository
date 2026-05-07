package org.globalart.astrum.repository.p2.internal.util;

import java.util.Map;

import org.globalart.astrum.repository.p2.internal.AssetKind;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenParser;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

public class P2PathUtilsTest
{
  private static final String SITE = "945d37142fbf9b4c2885a99ebcf353ab852479348e7e86446557f7f05fe87e83";

  @Test
  public void routesMetadataAtTheRootAndInSites() {
    for (String prefix : new String[]{"", "/" + SITE}) {
      assertEquals(AssetKind.P2_INDEX, kind(prefix + "/p2.index"));
      assertEquals(AssetKind.COMPOSITE_CONTENT, kind(prefix + "/compositeContent.jar"));
      assertEquals(AssetKind.COMPOSITE_CONTENT, kind(prefix + "/compositeContent.xml"));
      assertEquals(AssetKind.COMPOSITE_ARTIFACTS, kind(prefix + "/compositeArtifacts.jar"));
      assertEquals(AssetKind.COMPOSITE_ARTIFACTS, kind(prefix + "/compositeArtifacts.xml"));
      assertEquals(AssetKind.CONTENT_METADATA, kind(prefix + "/content.jar"));
      assertEquals(AssetKind.CONTENT_METADATA, kind(prefix + "/content.xml.xz"));
      assertEquals(AssetKind.CONTENT_METADATA, kind(prefix + "/content.xml"));
      assertEquals(AssetKind.ARTIFACTS_METADATA, kind(prefix + "/artifacts.jar"));
      assertEquals(AssetKind.ARTIFACTS_METADATA, kind(prefix + "/artifacts.xml.xz"));
      assertEquals(AssetKind.ARTIFACTS_METADATA, kind(prefix + "/artifacts.xml"));
    }
  }

  @Test
  public void routesBundlesFeaturesAndBinaries() {
    Map<String, String> tokens = tokens("/plugins/org.eclipse.swt.cocoa.macosx.x86_64_3.124.0.v20231113-1355.jar");
    assertEquals("org.eclipse.swt.cocoa.macosx.x86_64", tokens.get("name"));
    assertEquals("3.124.0.v20231113-1355", tokens.get("version"));
    assertEquals("jar", tokens.get("extension"));

    tokens = tokens("/" + SITE + "/plugins/com.google.code.atinject.tck_1.1.0.v20160901-1938.jar.pack.gz");
    assertEquals(SITE, tokens.get("site"));
    assertEquals("com.google.code.atinject.tck", tokens.get("name"));
    assertEquals("1.1.0.v20160901-1938", tokens.get("version"));
    assertEquals("jar.pack.gz", tokens.get("extension"));

    tokens = tokens("/features/org.eclipse.m2e.feature_2.7.0.20240218-1233.jar");
    assertEquals("features", tokens.get("dir"));
    assertEquals("org.eclipse.m2e.feature", tokens.get("name"));
    assertEquals("2.7.0.20240218-1233", tokens.get("version"));

    tokens = tokens("/binary/org.eclipse.equinox.executable_root.cocoa.macosx.aarch64_3.8.2400.v20240213-1244");
    assertEquals("binary", tokens.get("dir"));
    assertEquals("org.eclipse.equinox.executable_root.cocoa.macosx.aarch64", tokens.get("name"));
    assertEquals("3.8.2400.v20240213-1244", tokens.get("version"));

    assertEquals(AssetKind.BUNDLE, kind("/plugins/org.junit_4.13.2.v20230809-1000.jar"));
    assertEquals(AssetKind.BINARY_BUNDLE, kind("/binary/org.eclipse.platform.ide.executable.gtk_4.31.0.v20240229-1022"));
  }

  @Test
  public void rejectsOtherPaths() {
    assertNull(kind("/site.xml"));
    assertNull(kind("/index.html"));
    assertNull(kind("/plugins/readme.txt"));
    assertNull(kind("/abc/content.jar"));
    assertNull(kind("/" + SITE + "/" + SITE + "/content.jar"));
  }

  @Test
  public void buildsAttributesFromPaths() {
    String path = "/plugins/org.eclipse.swt.cocoa.macosx.x86_64_3.124.0.v20231113-1355.jar";
    P2Attributes attributes = P2PathUtils.toP2Attributes(path, state(path));
    assertEquals("org.eclipse.swt.cocoa.macosx.x86_64", attributes.getComponentName());
    assertEquals("3.124.0.v20231113-1355", attributes.getComponentVersion());
    assertEquals("jar", attributes.getExtension());
    assertEquals("org.eclipse.swt.cocoa.macosx.x86_64_3.124.0.v20231113-1355.jar", attributes.getFileName());
    assertEquals(path, attributes.getPath());

    path = "/binary/org.eclipse.platform.ide.executable.gtk_4.31.0.v20240229-1022";
    attributes = P2PathUtils.toP2AttributesBinary(path, state(path));
    assertEquals("org.eclipse.platform.ide.executable.gtk", attributes.getComponentName());
    assertEquals("org.eclipse.platform.ide.executable.gtk", attributes.getPluginName());
    assertEquals("4.31.0.v20240229-1022", attributes.getComponentVersion());
  }

  private static AssetKind kind(final String path) {
    for (String prefix : new String[]{P2PathUtils.SITE_PREFIX, ""}) {
      for (Map.Entry<String, AssetKind> entry : P2PathUtils.patterns(prefix).entrySet()) {
        if (new TokenParser(entry.getKey()).parse(path) != null) {
          return entry.getValue();
        }
      }
    }
    return null;
  }

  private static Map<String, String> tokens(final String path) {
    for (String prefix : new String[]{P2PathUtils.SITE_PREFIX, ""}) {
      for (String pattern : P2PathUtils.patterns(prefix).keySet()) {
        Map<String, String> tokens = new TokenParser(pattern).parse(path);
        if (tokens != null) {
          return tokens;
        }
      }
    }
    throw new AssertionError("No route for " + path);
  }

  private static TokenMatcher.State state(final String path) {
    Map<String, String> tokens = tokens(path);
    return new TokenMatcher.State()
    {
      @Override
      public String pattern() {
        return "";
      }

      @Override
      public Map<String, String> getTokens() {
        return tokens;
      }
    };
  }
}
