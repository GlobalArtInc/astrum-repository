package org.globalart.astrum.repository.gitlfs.internal;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;

class GitLfsObjectsTest
{
  static final String OID = "4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393";

  @Test
  void acceptsOnlyLowerCaseSha256Ids() {
    assertThat(GitLfsObjects.isValidOid(OID), is(true));
    assertThat(GitLfsObjects.isValidOid(OID.toUpperCase()), is(false));
    assertThat(GitLfsObjects.isValidOid(OID.substring(1)), is(false));
    assertThat(GitLfsObjects.isValidOid(OID + "0"), is(false));
    assertThat(GitLfsObjects.isValidOid("../" + OID.substring(3)), is(false));
    assertThat(GitLfsObjects.isValidOid(null), is(false));
  }

  @Test
  void storesObjectsUnderTheirIdPrefix() {
    assertThat(GitLfsObjects.assetPath(OID), is("/objects/4d/7a/" + OID));
    assertThrows(IllegalArgumentException.class, () -> GitLfsObjects.assetPath("abc"));
  }

  @Test
  void readsTheIdBackFromAnAssetPath() {
    assertThat(GitLfsObjects.oidFromAssetPath("/objects/4d/7a/" + OID), is(OID));
    assertThat(GitLfsObjects.oidFromAssetPath("/objects/7a/4d/" + OID), is(nullValue()));
    assertThat(GitLfsObjects.oidFromAssetPath("/objects/" + OID), is(nullValue()));
    assertThat(GitLfsObjects.oidFromAssetPath("/info/lfs/objects/4d/7a/" + OID), is(nullValue()));
    assertThat(GitLfsObjects.oidFromAssetPath(null), is(nullValue()));
  }
}
