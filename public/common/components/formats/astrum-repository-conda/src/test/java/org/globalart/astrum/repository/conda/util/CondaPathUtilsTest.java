package org.globalart.astrum.repository.conda.util;

import java.util.Map;

import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.util.CondaPathUtils.PackagePath;
import org.globalart.astrum.repository.view.matchers.token.TokenParser;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

public class CondaPathUtilsTest
{
  @Test
  public void parsesPackagePathsWithDashesInNames() {
    PackagePath packagePath = CondaPathUtils.parsePackagePath("/noarch/python-dateutil-2.9.0-pyhd8ed1ab_0.conda")
        .orElseThrow();
    assertEquals("/noarch", packagePath.getSubdirPath());
    assertEquals("noarch", packagePath.getSubdir());
    assertEquals("python-dateutil", packagePath.getName());
    assertEquals("2.9.0", packagePath.getVersion());
    assertEquals("pyhd8ed1ab_0", packagePath.getBuild());
    assertEquals(AssetKind.ARCH_CONDA_PACKAGE, packagePath.getAssetKind());
    assertEquals("/noarch/python-dateutil-2.9.0-pyhd8ed1ab_0.conda", packagePath.getPath());
  }

  @Test
  public void parsesPackagePathsBelowChannelPrefix() {
    PackagePath packagePath = CondaPathUtils.parsePackagePath("conda-forge/osx-arm64/zlib-1.3.1-h8359307_2.tar.bz2")
        .orElseThrow();
    assertEquals("/conda-forge/osx-arm64", packagePath.getSubdirPath());
    assertEquals("osx-arm64", packagePath.getSubdir());
    assertEquals("zlib", packagePath.getName());
    assertEquals(AssetKind.ARCH_TAR_PACKAGE, packagePath.getAssetKind());
  }

  @Test
  public void rejectsNonPackagePaths() {
    assertFalse(CondaPathUtils.parsePackagePath("/osx-arm64/repodata.json").isPresent());
    assertFalse(CondaPathUtils.parsePackagePath("/zlib-1.3.1-0.conda").isPresent());
    assertFalse(CondaPathUtils.parsePackagePath("/linux-64/zlib.conda").isPresent());
  }

  @Test
  public void validatesSubdirs() {
    assertTrue(CondaPathUtils.isValidSubdir("noarch"));
    assertTrue(CondaPathUtils.isValidSubdir("linux-64"));
    assertTrue(CondaPathUtils.isValidSubdir("osx-arm64"));
    assertTrue(CondaPathUtils.isValidSubdir("emscripten-wasm32"));
    assertFalse(CondaPathUtils.isValidSubdir("shards"));
    assertFalse(CondaPathUtils.isValidSubdir("Linux-64"));
    assertFalse(CondaPathUtils.isValidSubdir(null));
  }

  @Test
  public void routePatternsExtractTokens() {
    Map<String, String> tokens = new TokenParser(CondaPathUtils.pattern(AssetKind.ARCH_CONDA_PACKAGE))
        .parse("/conda-forge/linux-64/ca-certificates-2024.8.30-hbcca054_0.conda");
    assertEquals("/conda-forge", tokens.get("path"));
    assertEquals("linux-64", tokens.get("subdir"));
    assertEquals("ca-certificates", tokens.get("name"));
    assertEquals("2024.8.30", tokens.get("version"));
    assertEquals("hbcca054_0", tokens.get("build"));

    tokens = new TokenParser(CondaPathUtils.pattern(AssetKind.ARCH_REPODATA_JSON)).parse("/noarch/repodata.json");
    assertEquals("", tokens.get("path"));
    assertEquals("noarch", tokens.get("subdir"));

    assertNull(new TokenParser(CondaPathUtils.pattern(AssetKind.ARCH_TAR_PACKAGE)).parse("/noarch/repodata.json"));
    assertTrue(new TokenParser(CondaPathUtils.pattern(AssetKind.ARCH_SHARD))
        .parse("/osx-arm64/0a1b2c.msgpack.zst") != null);
    assertNull(new TokenParser(CondaPathUtils.pattern(AssetKind.ARCH_SHARD))
        .parse("/osx-arm64/repodata_shards.msgpack.zst"));
    assertTrue(new TokenParser(CondaPathUtils.pattern(AssetKind.CHANNEL_DATA_JSON))
        .parse("/channeldata.json") != null);
  }
}
