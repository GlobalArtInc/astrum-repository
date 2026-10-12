package org.globalart.astrum.repository.swift.internal;

import java.util.List;
import java.util.Optional;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class ManifestsTest
{
  @Test
  void recognisesManifestFilenames() {
    assertThat(Manifests.isManifest("Package.swift"), is(true));
    assertThat(Manifests.isManifest("Package@swift-5.swift"), is(true));
    assertThat(Manifests.isManifest("Package@swift-5.9.1.swift"), is(true));
    assertThat(Manifests.isManifest("Package@swift-.swift"), is(false));
    assertThat(Manifests.isManifest("package.swift"), is(false));
    assertThat(Manifests.isManifest("Sources/Package.swift"), is(false));
    assertThat(Manifests.variantVersion("Package@swift-4.2.swift"), is(Optional.of("4.2")));
    assertThat(Manifests.variantVersion("Package.swift"), is(Optional.empty()));
  }

  @Test
  void readsToolsVersions() {
    assertThat(Manifests.toolsVersion("// swift-tools-version:5.9\nimport PackageDescription"), is(Optional.of("5.9")));
    assertThat(Manifests.toolsVersion("// swift-tools-version: 5.7.1\n"), is(Optional.of("5.7.1")));
    assertThat(Manifests.toolsVersion("//swift-tools-version:6.0;(experimentalFeatures)"), is(Optional.of("6.0")));
    assertThat(Manifests.toolsVersion("﻿// Swift-Tools-Version:5.3"), is(Optional.of("5.3")));
    assertThat(Manifests.toolsVersion("import PackageDescription\n// swift-tools-version:5.9"), is(Optional.empty()));
    assertThat(Manifests.toolsVersionOrVariant("Package@swift-4.2.swift", "import Foundation"), is("4.2"));
  }

  @Test
  void resolvesSwiftVersionsLikeTheClient() {
    List<String> variants = List.of("Package@swift-4.swift", "Package@swift-4.2.swift", "Package@swift-5.7.1.swift");
    assertThat(Manifests.resolve(variants, "4.2"), is(Optional.of("Package@swift-4.2.swift")));
    assertThat(Manifests.resolve(variants, "4.2.0"), is(Optional.of("Package@swift-4.2.swift")));
    assertThat(Manifests.resolve(variants, "4.0.0"), is(Optional.of("Package@swift-4.swift")));
    assertThat(Manifests.resolve(variants, "4"), is(Optional.of("Package@swift-4.swift")));
    assertThat(Manifests.resolve(variants, "5.7.1"), is(Optional.of("Package@swift-5.7.1.swift")));
    assertThat(Manifests.resolve(variants, "5.7"), is(Optional.empty()));
    assertThat(Manifests.resolve(variants, "latest"), is(Optional.empty()));
    assertThat(Manifests.resolve(variants, null), is(Optional.empty()));
  }
}
