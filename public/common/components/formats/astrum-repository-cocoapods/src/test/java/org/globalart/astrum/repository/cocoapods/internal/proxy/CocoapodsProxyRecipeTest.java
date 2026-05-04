package org.globalart.astrum.repository.cocoapods.internal.proxy;

import java.util.Arrays;
import java.util.Map;

import org.globalart.astrum.repository.view.matchers.token.TokenParser;

import org.junit.Test;

import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.FILENAME_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.PACKAGE_NAME_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.PACKAGE_VERSION_KEY;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

public class CocoapodsProxyRecipeTest
{
  @Test
  public void cdnMetadataPatterns() {
    assertTrue(matchesMetadata("/CocoaPods-version.yml"));
    assertTrue(matchesMetadata("/deprecated_podspecs.txt"));
    assertTrue(matchesMetadata("/all_pods.txt"));
    assertTrue(matchesMetadata("/all_pods_versions_d_a_2.txt"));
    assertTrue(matchesMetadata("/all_pods_versions_0_f.txt"));
    assertTrue(matchesMetadata("/AlgoliaSearch.yml"));
    assertFalse(matchesMetadata("/nested/CocoaPods-version.yml"));
    assertFalse(matchesMetadata("/all_pods_versions_x_y_z.txt"));
  }

  @Test
  public void specPattern() {
    TokenParser parser = new TokenParser(CocoapodsProxyRecipe.SPEC_PATTERN);
    assertNotNull(parser.parse("/Specs/d/a/2/Alamofire/5.10.2/Alamofire.podspec.json"));
    assertNull(parser.parse("/Specs/d/a/2/Alamofire/5.10.2/Alamofire.podspec"));
  }

  @Test
  public void podPattern() {
    Map<String, String> tokens = new TokenParser(CocoapodsProxyRecipe.POD_PATTERN)
        .parse("/pods/Alamofire/5.10.2/5.10.2.tar.gz");
    assertEquals("Alamofire", tokens.get(PACKAGE_NAME_KEY));
    assertEquals("5.10.2", tokens.get(PACKAGE_VERSION_KEY));
    assertEquals("5.10.2.tar.gz", tokens.get(FILENAME_KEY));
    assertNull(new TokenParser(CocoapodsProxyRecipe.POD_PATTERN).parse("/pods/Alamofire/5.10.2/x/y.tar.gz"));
  }

  private static boolean matchesMetadata(final String path) {
    return Arrays.stream(CocoapodsProxyRecipe.CDN_METADATA_PATTERNS)
        .anyMatch(pattern -> new TokenParser(pattern).parse(path) != null);
  }
}
