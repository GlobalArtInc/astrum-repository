
package org.globalart.astrum.repository.maven.internal.utils;

import java.util.Map;

import org.globalart.astrum.repository.maven.MavenPath.Coordinates;

import org.junit.Test;

import static org.apache.commons.lang3.StringUtils.EMPTY;
import static org.hamcrest.collection.IsMapContaining.hasEntry;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.repository.maven.MavenPath.SignatureType.GPG;
import static org.globalart.astrum.repository.maven.internal.utils.MavenVariableResolverAdapterUtil.createCoordinateMap;

public class MavenVariableResolverAdapterUtilTest

{
  @Test
  public void shouldCopyCoordinatesToMap() {
    Coordinates coordinates = new Coordinates(false, "org.mockito", "mockito-core",
        "3.24", 3600L, 100, "3.24", "test", ".jar", GPG);

    Map<String, String> map = createCoordinateMap(coordinates);

    assertThat(map, hasEntry("groupId", "org.mockito"));
    assertThat(map, hasEntry("artifactId", "mockito-core"));
    assertThat(map, hasEntry("version", "3.24"));
    assertThat(map, hasEntry("extension", ".jar"));
    assertThat(map, hasEntry("classifier", "test"));
  }

  @Test
  public void classifierShouldBeEmptyStringWhenNotSet() {
    Coordinates coordinates = new Coordinates(false, "org.mockito", "mockito-core",
        "3.24", 3600L, 100, "3.24", null, ".jar", GPG);

    Map<String, String> map = createCoordinateMap(coordinates);

    assertThat(map, hasEntry("classifier", EMPTY));
  }
}
