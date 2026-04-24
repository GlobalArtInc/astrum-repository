package org.globalart.astrum.repository.alpine.internal.apk;

import java.io.ByteArrayInputStream;
import java.security.Signature;
import java.time.Instant;
import java.util.List;
import java.util.Map;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

public class ApkIndexArchiveTest
{
  private static final String INDEX = "C:Q1aaaaaaaaaaaaaaaaaaaaaaaaaaa=\n"
      + "P:alpha\n"
      + "V:1.0-r0\n"
      + "A:x86_64\n"
      + "S:10\n"
      + "I:20\n"
      + "T:Alpha\n"
      + "U:\n"
      + "L:MIT\n"
      + "\n"
      + "C:Q1bbbbbbbbbbbbbbbbbbbbbbbbbbb=\n"
      + "P:beta\n"
      + "V:2.0-r1\n"
      + "A:x86_64\n"
      + "S:11\n"
      + "I:21\n"
      + "T:Beta\n"
      + "U:\n"
      + "L:MIT\n"
      + "D:alpha\n"
      + "\n";

  @Test
  public void parsesAndRendersEntries() {
    List<ApkIndexEntry> entries = ApkIndexEntry.parse(INDEX);

    assertEquals(2, entries.size());
    assertEquals("alpha-1.0-r0", entries.get(0).key());
    assertEquals("beta-2.0-r1", entries.get(1).key());
    assertEquals(INDEX, ApkIndexEntry.render(entries));
  }

  @Test
  public void buildsSignedIndexLikeAbuild() throws Exception {
    TestKeys keys = TestKeys.generate();
    ApkSigningKey key = ApkSigningKey.parse("alpine-hosted", keys.pkcs8(), null);
    byte[] archive = ApkIndexArchive.build("alpine-hosted", ApkIndexEntry.parse(INDEX), key, Instant.now());

    GzipMembers members = new GzipMembers(new ByteArrayInputStream(archive), Long.MAX_VALUE);
    GzipMembers.Member signatureSegment = members.next();
    GzipMembers.Member indexSegment = members.next();
    assertNull(members.next());

    assertEquals(1024, signatureSegment.inflated().length);
    Map<String, byte[]> signatureEntries = ApkPackageParser.tarEntries(signatureSegment.inflated());
    byte[] signature = signatureEntries.get(".SIGN.RSA.alpine-hosted.rsa.pub");
    Signature verifier = Signature.getInstance("SHA1withRSA");
    verifier.initVerify(keys.publicKey());
    verifier.update(indexSegment.raw());
    assertTrue(verifier.verify(signature));

    ApkIndexArchive.Contents contents = ApkIndexArchive.read(new ByteArrayInputStream(archive));
    assertEquals("alpine-hosted", contents.description());
    assertEquals(".SIGN.RSA.alpine-hosted.rsa.pub", contents.signature());
    assertEquals(INDEX, contents.apkindex());
  }

  @Test
  public void buildsUnsignedIndex() throws Exception {
    byte[] archive = ApkIndexArchive.build("repo", ApkIndexEntry.parse(INDEX), null, Instant.now());

    ApkIndexArchive.Contents contents = ApkIndexArchive.read(new ByteArrayInputStream(archive));
    assertNull(contents.signature());
    assertEquals(2, contents.entries().size());
  }
}
