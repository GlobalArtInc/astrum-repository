package org.globalart.astrum.repository.terraform.internal.hosted;

import java.util.Map;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.internal.signing.Pgp;
import org.globalart.astrum.repository.terraform.internal.signing.PgpTestKeys;
import org.globalart.astrum.repository.terraform.internal.signing.Sha256Sums;

import org.junit.BeforeClass;
import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThatCode;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

public class ProviderReleaseTest
{
  private static final ProviderCoordinates DEMO = new ProviderCoordinates("acme", "demo", "2.0.0");

  private static final String LINUX = "terraform-provider-demo_2.0.0_linux_amd64.zip";

  private static final String DARWIN = "terraform-provider-demo_2.0.0_darwin_arm64.zip";

  private static final String LINUX_SHA = "a".repeat(64);

  private static final String DARWIN_SHA = "b".repeat(64);

  private static final ProviderRelease.Files NOTHING = new ProviderRelease.Files(Map.of(), null, null, null);

  private static Pgp.SigningKey publisher;

  private static Pgp.SigningKey other;

  @BeforeClass
  public static void generateKeys() throws Exception {
    publisher = Pgp.readSigningKey(PgpTestKeys.secretKeyRing("Publisher <publisher@example.com>", "p"), "p");
    other = Pgp.readSigningKey(PgpTestKeys.secretKeyRing("Other <other@example.com>", "o"), "o");
  }

  @Test
  public void acceptsPackagesTheChecksumsOfTheSameUploadList() {
    byte[] sums = sums(Map.of(LINUX, LINUX_SHA, DARWIN, DARWIN_SHA));

    assertThatCode(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(LINUX, LINUX_SHA, DARWIN, DARWIN_SHA), sums, null, null), NOTHING, null))
        .doesNotThrowAnyException();
  }

  @Test
  public void refusesTheWholeUploadWhenOnePackageDoesNotMatch() {
    byte[] sums = sums(Map.of(LINUX, LINUX_SHA, DARWIN, LINUX_SHA));

    assertThatThrownBy(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(LINUX, LINUX_SHA, DARWIN, DARWIN_SHA), sums, null, null), NOTHING, null))
        .isInstanceOf(InvalidContentException.class)
        .hasMessageContaining(DARWIN + " does not match");
  }

  @Test
  public void refusesAPackageTheStoredChecksumsDoNotList() {
    ProviderRelease.Files stored = new ProviderRelease.Files(Map.of(), sums(Map.of(LINUX, LINUX_SHA)), null, null);

    assertThatThrownBy(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(DARWIN, DARWIN_SHA), null, null, null), stored, null))
        .isInstanceOf(InvalidContentException.class)
        .hasMessageContaining("does not list " + DARWIN);
  }

  @Test
  public void checksNewChecksumsAgainstStoredPackagesTheUploadDoesNotReplace() {
    ProviderRelease.Files stored = new ProviderRelease.Files(Map.of(LINUX, LINUX_SHA), null, null, null);
    byte[] sums = sums(Map.of(LINUX, DARWIN_SHA));

    assertThatThrownBy(() -> ProviderRelease.check(DEMO, new ProviderRelease.Files(Map.of(), sums, null, null),
        stored, null))
        .isInstanceOf(InvalidContentException.class)
        .hasMessageContaining("does not match the stored " + LINUX);
    assertThatCode(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(LINUX, DARWIN_SHA), sums, null, null), stored, null))
        .doesNotThrowAnyException();
  }

  @Test
  public void checksTheSignatureWithTheKeyOfTheSameUpload() {
    byte[] sums = sums(Map.of(LINUX, LINUX_SHA));
    byte[] signature = Pgp.sign(publisher, sums);

    assertThatCode(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(LINUX, LINUX_SHA), sums, signature, publicKey(publisher)),
        new ProviderRelease.Files(Map.of(), null, Pgp.sign(other, sums), publicKey(other)), null))
        .doesNotThrowAnyException();
    assertThatThrownBy(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(), sums, signature, publicKey(other)), NOTHING, null))
        .isInstanceOf(InvalidContentException.class)
        .hasMessageContaining("did not sign");
  }

  @Test
  public void checksAnUploadedSignatureWithTheStoredAndTheRepositoryKey() {
    byte[] sums = sums(Map.of(LINUX, LINUX_SHA));
    ProviderRelease.Files stored = new ProviderRelease.Files(Map.of(LINUX, LINUX_SHA), sums, null, publicKey(other));

    assertThatThrownBy(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(), null, Pgp.sign(publisher, sums), null), stored, null))
        .isInstanceOf(InvalidContentException.class)
        .hasMessageContaining("by a known key");
    assertThatCode(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(), null, Pgp.sign(publisher, sums), null), stored, publicKey(publisher)))
        .doesNotThrowAnyException();
  }

  @Test
  public void leavesPackagesUncheckedWithoutChecksumsOfThePublisher() {
    assertThatCode(() -> ProviderRelease.check(DEMO,
        new ProviderRelease.Files(Map.of(LINUX, LINUX_SHA), null, null, null), NOTHING, null))
        .doesNotThrowAnyException();
  }

  private static byte[] sums(final Map<String, String> digests) {
    return Sha256Sums.of(digests).toBytes();
  }

  private static Pgp.PublicKey publicKey(final Pgp.SigningKey key) {
    return new Pgp.PublicKey(key.keyId(), key.armoredPublicKey());
  }
}
