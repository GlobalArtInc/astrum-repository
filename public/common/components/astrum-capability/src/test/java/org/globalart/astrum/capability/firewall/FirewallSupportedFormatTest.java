
package org.globalart.astrum.capability.firewall;

import org.junit.Test;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;

public class FirewallSupportedFormatTest
{

  @Test
  public void testEnumValues() {
    FirewallSupportedFormat[] expectedValues = {
        FirewallSupportedFormat.R,
        FirewallSupportedFormat.GO,
        FirewallSupportedFormat.P2,
        FirewallSupportedFormat.RUBYGEMS,
        FirewallSupportedFormat.NPM,
        FirewallSupportedFormat.YUM,
        FirewallSupportedFormat.PYPI,
        FirewallSupportedFormat.CARGO,
        FirewallSupportedFormat.CONAN,
        FirewallSupportedFormat.CONDA,
        FirewallSupportedFormat.NUGET,
        FirewallSupportedFormat.MAVEN2,
        FirewallSupportedFormat.COMPOSER,
        FirewallSupportedFormat.COCOAPODS,
        FirewallSupportedFormat.HUGGINGFACE,
        FirewallSupportedFormat.DOCKER,
        FirewallSupportedFormat.RAW
    };
    assertArrayEquals(expectedValues, FirewallSupportedFormat.values());
  }

  @Test
  public void testGetValue() {
    assertEquals("r", FirewallSupportedFormat.R.getValue());
    assertEquals("go", FirewallSupportedFormat.GO.getValue());
    assertEquals("p2", FirewallSupportedFormat.P2.getValue());
    assertEquals("rubygems", FirewallSupportedFormat.RUBYGEMS.getValue());
    assertEquals("npm", FirewallSupportedFormat.NPM.getValue());
    assertEquals("yum", FirewallSupportedFormat.YUM.getValue());
    assertEquals("pypi", FirewallSupportedFormat.PYPI.getValue());
    assertEquals("cargo", FirewallSupportedFormat.CARGO.getValue());
    assertEquals("conan", FirewallSupportedFormat.CONAN.getValue());
    assertEquals("conda", FirewallSupportedFormat.CONDA.getValue());
    assertEquals("nuget", FirewallSupportedFormat.NUGET.getValue());
    assertEquals("maven2", FirewallSupportedFormat.MAVEN2.getValue());
    assertEquals("composer", FirewallSupportedFormat.COMPOSER.getValue());
    assertEquals("cocoapods", FirewallSupportedFormat.COCOAPODS.getValue());
    assertEquals("huggingface", FirewallSupportedFormat.HUGGINGFACE.getValue());
    assertEquals("docker", FirewallSupportedFormat.DOCKER.getValue());
  }

  @Test
  public void testGetValues() {
    String[] expectedValues = {
        "r", "go", "p2", "rubygems", "npm", "yum", "pypi", "cargo", "conan", "conda", "nuget", "maven2",
        "composer", "cocoapods", "huggingface", "docker", "raw"
    };
    assertArrayEquals(expectedValues, FirewallSupportedFormat.getValues());
  }
}
