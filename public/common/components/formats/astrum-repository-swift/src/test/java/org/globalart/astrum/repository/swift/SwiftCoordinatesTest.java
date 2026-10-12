package org.globalart.astrum.repository.swift;

import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;

class SwiftCoordinatesTest
{
  @Test
  void validatesScopes() {
    for (String scope : List.of("mona", "MONA", "m", "a-b", "a1-b2-c3", "a".repeat(39))) {
      assertThat(scope, SwiftCoordinates.isValidScope(scope), is(true));
    }
    for (String scope : List.of("", "-mona", "mona-", "mo--na", "mo_na", "mo.na", "a".repeat(40), "../x")) {
      assertThat(scope, SwiftCoordinates.isValidScope(scope), is(false));
    }
  }

  @Test
  void validatesNames() {
    for (String name : List.of("LinkedList", "linked-list", "linked_list", "a1_b-c", "a".repeat(100))) {
      assertThat(name, SwiftCoordinates.isValidName(name), is(true));
    }
    for (String name : List.of("", "_x", "x_", "x-", "a__b", "a-_b", "a.b", "a".repeat(101))) {
      assertThat(name, SwiftCoordinates.isValidName(name), is(false));
    }
  }

  @Test
  void buildsPaths() {
    SwiftCoordinates coordinates = new SwiftCoordinates("mona", "LinkedList", "1.1.1");
    assertThat(coordinates.getIdentifier(), is("mona.LinkedList"));
    assertThat(coordinates.getReleasesPath(), is("/mona/LinkedList.json"));
    assertThat(coordinates.getReleasePath(), is("/mona/LinkedList/1.1.1"));
    assertThat(coordinates.getReleaseMetadataPath(), is("/mona/LinkedList/1.1.1.json"));
    assertThat(coordinates.getArchivePath(), is("/mona/LinkedList/1.1.1.zip"));
    assertThat(coordinates.getManifestPath("Package@swift-5.9.swift"), is("/mona/LinkedList/1.1.1/Package@swift-5.9.swift"));
    assertThat(coordinates.getArchiveFilename(), is("LinkedList-1.1.1.zip"));
    assertThat(coordinates.samePackage("MONA", "linkedlist"), is(true));
    assertThat(coordinates.getLowerName(), is("linkedlist"));
  }

  @Test
  void parsesIdentifiers() {
    assertThat(SwiftCoordinates.fromIdentifier("mona.LinkedList", "1.0.0"),
        is(new SwiftCoordinates("mona", "LinkedList", "1.0.0")));
    assertThat(SwiftCoordinates.fromIdentifier("mona", null), is(nullValue()));
    assertThat(SwiftCoordinates.fromIdentifier("mona.Linked.List", null), is(nullValue()));
    assertThat(SwiftCoordinates.fromIdentifier("mona.LinkedList", "1.0"), is(nullValue()));
  }

  @Test
  void rejectsInvalidCoordinates() {
    assertThrows(IllegalArgumentException.class, () -> new SwiftCoordinates("../x", "a", null));
    assertThrows(IllegalArgumentException.class, () -> new SwiftCoordinates("x", "a", "latest"));
    assertThrows(IllegalArgumentException.class, () -> new SwiftCoordinates("x", "a", null).getArchivePath());
  }
}
