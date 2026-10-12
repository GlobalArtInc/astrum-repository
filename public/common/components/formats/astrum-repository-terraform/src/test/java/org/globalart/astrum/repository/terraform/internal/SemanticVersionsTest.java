package org.globalart.astrum.repository.terraform.internal;

import java.util.ArrayList;
import java.util.List;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class SemanticVersionsTest
{
  @Test
  public void ordersBySemanticVersionPrecedence() {
    List<String> versions = new ArrayList<>(List.of("1.10.0", "1.2.0", "1.0.0", "1.0.0-rc.1", "1.0.0-beta.11",
        "1.0.0-beta.2", "1.0.0-alpha", "2.0.0", "1.0.0-alpha.1"));

    versions.sort(SemanticVersions.ORDER);

    assertThat(versions).containsExactly("1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-beta.2", "1.0.0-beta.11",
        "1.0.0-rc.1", "1.0.0", "1.2.0", "1.10.0", "2.0.0");
  }

  @Test
  public void ignoresBuildMetadata() {
    assertThat(SemanticVersions.compare("1.0.0+build.1", "1.0.0+build.2")).isZero();
  }
}
