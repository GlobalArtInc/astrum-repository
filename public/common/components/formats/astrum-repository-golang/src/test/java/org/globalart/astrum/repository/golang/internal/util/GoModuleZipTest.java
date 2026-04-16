package org.globalart.astrum.repository.golang.internal.util;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.globalart.astrum.repository.InvalidContentException;

import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class GoModuleZipTest
{
  @Test
  public void readsModuleVersionAndGoMod() throws IOException {
    byte[] zip = zip(
        "example.com/Hello@v1.0.0/go.mod", "module example.com/Hello\n",
        "example.com/Hello@v1.0.0/hello.go", "package hello\n",
        "example.com/Hello@v1.0.0/sub/go.mod.txt", "not a go.mod\n");

    GoModuleZip moduleZip = GoModuleZip.read(new ByteArrayInputStream(zip));

    assertThat(moduleZip.module(), is(equalTo("example.com/Hello")));
    assertThat(moduleZip.version(), is(equalTo("v1.0.0")));
    assertThat(new String(moduleZip.goMod(), UTF_8), is(equalTo("module example.com/Hello\n")));
  }

  @Test
  public void goModIsOptional() throws IOException {
    byte[] zip = zip("example.com/hello@v1.0.0/hello.go", "package hello\n");

    assertThat(GoModuleZip.read(new ByteArrayInputStream(zip)).goMod(), is(nullValue()));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsEntriesOutsideModuleDirectory() throws IOException {
    GoModuleZip.read(new ByteArrayInputStream(zip("hello/go.mod", "module hello\n")));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsMixedModules() throws IOException {
    GoModuleZip.read(new ByteArrayInputStream(zip(
        "example.com/a@v1.0.0/a.go", "package a\n",
        "example.com/b@v1.0.0/b.go", "package b\n")));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsNonZipContent() {
    GoModuleZip.read(new ByteArrayInputStream("not a zip".getBytes(UTF_8)));
  }

  private static byte[] zip(final String... entries) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZipOutputStream zip = new ZipOutputStream(out)) {
      for (int i = 0; i < entries.length; i += 2) {
        zip.putNextEntry(new ZipEntry(entries[i]));
        zip.write(entries[i + 1].getBytes(UTF_8));
        zip.closeEntry();
      }
    }
    return out.toByteArray();
  }
}
