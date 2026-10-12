package org.globalart.astrum.repository.yum.internal;

import java.util.Optional;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

public class YumPathsTest
{
  @Test
  public void normalizesPaths() {
    assertEquals("/el9/x86_64/hello.rpm", YumPaths.normalize("el9//x86_64/hello.rpm"));
    assertEquals("/el9", YumPaths.normalize("/el9/"));
    assertEquals("/", YumPaths.normalize(""));
    assertThrows(IllegalArgumentException.class, () -> YumPaths.normalize("/el9/../etc/passwd"));
  }

  @Test
  public void findsTheRepodataBaseOfAPackage() {
    assertEquals(Optional.of(""), YumPaths.base("/hello-1-1.x86_64.rpm", 0));
    assertEquals(Optional.of(""), YumPaths.base("/Packages/h/hello-1-1.x86_64.rpm", 0));
    assertEquals(Optional.of("/el9/x86_64"), YumPaths.base("/el9/x86_64/Packages/hello-1-1.x86_64.rpm", 2));
    assertEquals(Optional.of("/el9/x86_64"), YumPaths.base("/el9/x86_64/hello-1-1.x86_64.rpm", 2));
    assertEquals(Optional.empty(), YumPaths.base("/el9/hello-1-1.x86_64.rpm", 2));
    assertEquals(Optional.empty(), YumPaths.base("/hello-1-1.x86_64.rpm", 1));
  }

  @Test
  public void findsTheBaseOfMetadata() {
    assertEquals(Optional.of(""), YumPaths.baseOfMetadata("/repodata/repomd.xml"));
    assertEquals(Optional.of("/el9/x86_64"), YumPaths.baseOfMetadata("/el9/x86_64/repodata/abc-primary.xml.gz"));
    assertEquals(Optional.empty(), YumPaths.baseOfMetadata("/el9/x86_64/hello.rpm"));
    assertEquals(Optional.empty(), YumPaths.baseOfMetadata("/repodata"));
  }

  @Test
  public void buildsMetadataPaths() {
    assertEquals("/repodata/repomd.xml", YumPaths.repomdPath(""));
    assertEquals("/el9/repodata/repomd.xml.asc", YumPaths.signaturePath("/el9"));
    assertEquals("Packages/hello.rpm", YumPaths.location("/el9", "/el9/Packages/hello.rpm"));
    assertEquals("hello.rpm", YumPaths.location("", "/hello.rpm"));
    assertEquals(2, YumPaths.depthOf("/el9/x86_64"));
    assertEquals(0, YumPaths.depthOf(""));
  }

  @Test
  public void recognizesReservedDirectories() {
    assertTrue(YumPaths.isInRepodata("/el9/repodata/repomd.xml"));
    assertFalse(YumPaths.isInRepodata("/el9/repodatas/repomd.xml"));
    assertTrue(YumPaths.isRpm("/el9/hello-1-1.x86_64.rpm"));
    assertFalse(YumPaths.isRpm("/el9/hello.rpm.asc"));
  }
}
