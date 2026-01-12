
package org.globalart.astrum.capability.firewall;

import java.util.stream.Stream;

public enum FirewallSupportedFormat
{
  R("r"),
  GO("go"),
  P2("p2"),
  RUBYGEMS("rubygems"),
  NPM("npm"),
  YUM("yum"),
  PYPI("pypi"),
  CARGO("cargo"),
  CONAN("conan"),
  CONDA("conda"),
  NUGET("nuget"),
  MAVEN2("maven2"),
  COMPOSER("composer"),
  COCOAPODS("cocoapods"),
  HUGGINGFACE("huggingface"),
  DOCKER("docker"),
  RAW("raw");

  private final String value;

  FirewallSupportedFormat(String value) {
    this.value = value;
  }

  public String getValue() {
    return value;
  }

  public static String[] getValues() {
    return Stream.of(FirewallSupportedFormat.values())
        .map(FirewallSupportedFormat::getValue)
        .toArray(String[]::new);
  }
}
